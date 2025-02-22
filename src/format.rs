use crate::error::Result;
use regex::Regex;
use std::fmt::{self, Write};
use lazy_static::lazy_static;

lazy_static! {
    static ref SEXAGESIMAL_RE: Regex = Regex::new(r"^%(\d+)\.(\d+)m$").unwrap();
    static ref NUMBER_RE: Regex = Regex::new(r"^\s*[-+]?\d*\.?\d*(?:[:; ]\d*\.?\d*)*\s*$").unwrap();
}

/// Number format specification
#[derive(Debug, Clone, PartialEq)]
pub enum NumberFormat {
    /// Printf-style format (e.g., "%.2f")
    Printf(String),
    /// Sexagesimal format (e.g., "%8.3m" for degrees)
    Sexagesimal {
        /// Total field width
        width: usize,
        /// Fraction precision
        precision: usize,
    },
}

impl fmt::Display for NumberFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberFormat::Printf(fmt_str) => write!(f, "{}", fmt_str),
            NumberFormat::Sexagesimal { width, precision } => write!(f, "%{}.{}m", width, precision),
        }
    }
}

impl NumberFormat {
    /// Parse a format string into a NumberFormat
    pub fn parse(format: &str) -> Result<Self> {
        // Check for sexagesimal format first
        if let Some(caps) = SEXAGESIMAL_RE.captures(format) {
            let width = caps[1].parse().map_err(|_| {
                crate::error::Error::Format("Invalid width in sexagesimal format".to_string())
            })?;
            let precision = caps[2].parse().map_err(|_| {
                crate::error::Error::Format("Invalid precision in sexagesimal format".to_string())
            })?;
            return Ok(NumberFormat::Sexagesimal { width, precision });
        }

        // Validate printf format
        if !format.contains('%') || format.matches('%').count() > 1 {
            return Err(crate::error::Error::Format("Invalid printf format".to_string()));
        }
        Ok(NumberFormat::Printf(format.to_string()))
    }

    /// Format a number according to the format specification
    pub fn format(&self, value: f64) -> Result<String> {
        match self {
            NumberFormat::Printf(fmt) => {
                let mut result = String::new();
                write!(&mut result, "{:width$}", value, width = fmt.len())
                    .map_err(|e| crate::error::Error::Format(e.to_string()))?;
                Ok(result)
            }
            NumberFormat::Sexagesimal { width, precision } => {
                format_sexagesimal(value, *width, *precision)
            }
        }
    }
}

/// Parse a number string that may be in sexagesimal format
pub fn parse_number(s: &str) -> Result<f64> {
    if !NUMBER_RE.is_match(s) {
        return Err(crate::error::Error::Format(format!("Invalid number format: {}", s)));
    }

    let s = s.trim(); // Remove leading/trailing whitespace
    let parts: Vec<&str> = s.split(|c| c == ':' || c == ';' || c == ' ').collect();
    
    // Parse the first part and remember its sign
    let first_part = parts[0].parse::<f64>().map_err(|_| {
        crate::error::Error::Format(format!("Invalid number: {}", parts[0]))
    })?;
    let sign = if first_part < 0.0 { -1.0 } else { 1.0 };
    let mut value = first_part.abs();

    // Apply the sign after all parts are processed
    let mut multiplier = 1.0/60.0;
    for part in parts.iter().skip(1) {
        if !part.is_empty() {
            value += part.parse::<f64>().map_err(|_| {
                crate::error::Error::Format(format!("Invalid number: {}", part))
            })? * multiplier;
        }
        multiplier /= 60.0;
    }
    
    Ok(value * sign)
}

/// Format a number in sexagesimal format
fn format_sexagesimal(value: f64, width: usize, precision: usize) -> Result<String> {
    let negative = value < 0.0;
    let abs_value = value.abs();
    let mut numeric = String::with_capacity(width);

    match precision {
        3 => {
            let minutes = (abs_value * 60.0).round() % 60.0;
            write!(numeric, "{:.0}:{:02.0}", abs_value.trunc(), minutes)?
        },
        5 => {
            let hours = abs_value.trunc();
            let minutes_float = (abs_value - hours) * 60.0;
            let minutes = minutes_float.trunc();
            let decimal_minutes = ((minutes_float - minutes) * 10.0).round();
            write!(numeric, "{:.0}:{:02.0}.{:1.0}", hours, minutes, decimal_minutes)?
        },
        6 => {
            let degrees = abs_value.trunc();
            let minutes_float = (abs_value - degrees) * 60.0;
            let minutes = minutes_float.trunc();
            let seconds = (minutes_float - minutes) * 60.0;
            write!(numeric, "{:.0}:{:02.0}:{:02.0}", degrees, minutes, seconds.round())?
        },
        8 => {
            let degrees = abs_value.trunc();
            let minutes_float = (abs_value - degrees) * 60.0;
            let minutes = minutes_float.trunc();
            let seconds = (minutes_float - minutes) * 60.0;
            write!(numeric, "{:.0}:{:02.0}:{:04.1}", degrees, minutes, seconds)?
        },
        9 => {
            let degrees = abs_value.trunc();
            let minutes_float = (abs_value - degrees) * 60.0;
            let minutes = minutes_float.trunc();
            let seconds = (minutes_float - minutes) * 60.0;
            write!(numeric, "{:.0}:{:02.0}:{:05.2}", degrees, minutes, seconds)?
        },
        _ => return Err(crate::error::Error::Format("Invalid sexagesimal precision".to_string())),
    }

    // For negative numbers, just add the minus sign
    if negative {
        return Ok(format!("-{}", numeric));
    }

    // For positive numbers, add correct number of spaces based on width and number of digits
    let hours = abs_value.trunc();
    let spaces = if width == 7 {
        if hours >= 100.0 {
            " "
        } else if hours >= 10.0 {
            "  "
        } else {
            "   "
        }
    } else {
        if hours >= 10.0 {
            " "
        } else {
            "  "
        }
    };

    Ok(format!("{}{}", spaces, numeric))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_format_parse() {
        assert!(matches!(
            NumberFormat::parse("%8.3m").unwrap(),
            NumberFormat::Sexagesimal { width: 8, precision: 3 }
        ));
        assert!(matches!(
            NumberFormat::parse("%.2f").unwrap(),
            NumberFormat::Printf(_)
        ));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("12.5").unwrap(), 12.5);
        assert_eq!(parse_number("-12:30").unwrap(), -12.5);
        assert_eq!(parse_number("12:30:00").unwrap(), 12.5);
        assert_eq!(parse_number("12 30").unwrap(), 12.5);
    }

    #[test]
    fn test_format_sexagesimal() {
        // Test width 3 (hours:minutes)
        assert_eq!(format_sexagesimal(123.75, 7, 3).unwrap(), " 123:45");
        assert_eq!(format_sexagesimal(-123.75, 7, 3).unwrap(), "-123:45");
        assert_eq!(format_sexagesimal(1.5, 7, 3).unwrap(), "   1:30");

        // Test width 5 (hours:minutes.decimal_minutes)
        assert_eq!(format_sexagesimal(1.5, 7, 5).unwrap(), "   1:30.0");
        assert_eq!(format_sexagesimal(1.525, 7, 5).unwrap(), "   1:31.5");
        assert_eq!(format_sexagesimal(-1.525, 7, 5).unwrap(), "-1:31.5");

        // Test width 6 (hours:minutes:seconds)
        assert_eq!(format_sexagesimal(1.5, 9, 6).unwrap(), "  1:30:00");
        assert_eq!(format_sexagesimal(12.5, 9, 6).unwrap(), " 12:30:00");
        assert_eq!(format_sexagesimal(-1.5, 9, 6).unwrap(), "-1:30:00");

        // Test width 8 (hours:minutes:seconds.decimal_seconds)
        assert_eq!(format_sexagesimal(1.508333, 9, 8).unwrap(), "  1:30:30.0");
        assert_eq!(format_sexagesimal(12.508333, 9, 8).unwrap(), " 12:30:30.0");
        assert_eq!(format_sexagesimal(-1.508333, 9, 8).unwrap(), "-1:30:30.0");

        // Test width 9 (hours:minutes:seconds.decimal_seconds)
        assert_eq!(format_sexagesimal(1.508333, 9, 9).unwrap(), "  1:30:30.00");
        assert_eq!(format_sexagesimal(12.508333, 9, 9).unwrap(), " 12:30:30.00");
        assert_eq!(format_sexagesimal(-1.508333, 9, 9).unwrap(), "-1:30:30.00");
    }
}
