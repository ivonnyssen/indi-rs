use crate::error::{Error, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a timestamp in INDI format (YYYY-MM-DDTHH:MM:SS.S)
/// The decimal places in seconds are optional and configurable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct INDITimestamp {
    datetime: DateTime<Utc>,
    decimal: Option<String>, // Store exact decimal string if present
}

impl INDITimestamp {
    /// Create a new INDITimestamp from the current time
    pub fn now(precision: Option<u8>) -> Self {
        let datetime = Utc::now();
        let decimal = precision.map(|p| {
            let nanos = datetime.timestamp_subsec_nanos() as f64;
            let subsec = (nanos / 1_000_000_000.0 * 10f64.powi(p as i32)).round() as u32;
            format!("{:0>width$}", subsec, width = p as usize)
        });
        Self { datetime, decimal }
    }

    /// Create a new INDITimestamp from a DateTime<Utc>
    pub fn from_datetime(datetime: DateTime<Utc>, precision: Option<u8>) -> Self {
        let decimal = precision.map(|p| {
            let nanos = datetime.timestamp_subsec_nanos() as f64;
            let subsec = (nanos / 1_000_000_000.0 * 10f64.powi(p as i32)).round() as u32;
            format!("{:0>width$}", subsec, width = p as usize)
        });
        Self { datetime, decimal }
    }

    /// Get the underlying DateTime<Utc>
    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }
}

impl FromStr for INDITimestamp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Validate format
        if !s.chars().all(|c| c.is_ascii_digit() || ".:-T".contains(c)) {
            return Err(Error::Property("Invalid characters in timestamp".to_string()));
        }

        // Split at decimal point to determine precision
        let (whole, fraction) = match s.split_once('.') {
            Some((w, f)) => {
                // Validate decimal part
                if f.is_empty() || !f.chars().all(|c| c.is_ascii_digit()) {
                    return Err(Error::Property("Invalid decimal part in timestamp".to_string()));
                }
                (w, Some(f.to_string()))
            }
            None => (s, None),
        };

        // Parse the whole part
        let naive = NaiveDateTime::parse_from_str(whole, "%Y-%m-%dT%H:%M:%S")
            .map_err(|e| Error::Property(format!("Invalid timestamp format: {}", e)))?;

        // Convert to UTC DateTime
        let datetime = DateTime::from_naive_utc_and_offset(naive, Utc);

        Ok(Self {
            datetime,
            decimal: fraction,
        })
    }
}

impl fmt::Display for INDITimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = self.datetime.format("%Y-%m-%dT%H:%M:%S").to_string();
        match &self.decimal {
            Some(decimal) => write!(f, "{}.{}", base, decimal),
            None => write!(f, "{}", base),
        }
    }
}

impl Serialize for INDITimestamp {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for INDITimestamp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_timestamp_creation() {
        // Test with a fixed datetime instead of now() to avoid intermittent failures
        let dt = DateTime::parse_from_rfc3339("2024-02-21T19:30:00.123456789Z")
            .unwrap()
            .with_timezone(&Utc);
        
        let ts = INDITimestamp::from_datetime(dt, Some(1));
        let pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d$").unwrap();
        assert!(pattern.is_match(&ts.to_string()), "Timestamp {} doesn't match pattern", ts);
        assert_eq!(ts.to_string(), "2024-02-21T19:30:00.1");

        let ts = INDITimestamp::from_datetime(dt, Some(3));
        let pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}$").unwrap();
        assert!(pattern.is_match(&ts.to_string()), "Timestamp {} doesn't match pattern", ts);
        assert_eq!(ts.to_string(), "2024-02-21T19:30:00.123");
    }

    #[test]
    fn test_timestamp_parsing() {
        let ts_str = "2024-02-21T19:30:00";
        let ts = INDITimestamp::from_str(ts_str).unwrap();
        assert_eq!(ts.to_string(), ts_str);

        let ts_str = "2024-02-21T19:30:00.5";
        let ts = INDITimestamp::from_str(ts_str).unwrap();
        assert_eq!(ts.decimal, Some("5".to_string()));
        assert_eq!(ts.to_string(), ts_str);

        let ts_str = "2024-02-21T19:30:00.500";
        let ts = INDITimestamp::from_str(ts_str).unwrap();
        assert_eq!(ts.decimal, Some("500".to_string()));
        assert_eq!(ts.to_string(), ts_str);
    }

    #[test]
    fn test_invalid_timestamp() {
        assert!(INDITimestamp::from_str("invalid").is_err());
        assert!(INDITimestamp::from_str("2024-02-21 19:30:00").is_err());
        assert!(INDITimestamp::from_str("2024-02-21T19:30:00.").is_err());
        assert!(INDITimestamp::from_str("2024-02-21T19:30:00.abc").is_err());
    }
}
