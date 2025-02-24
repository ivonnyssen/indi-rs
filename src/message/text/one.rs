use serde::{Deserialize, Serialize};

/// One text element used in new and set operations
/// 
/// According to the INDI protocol, this represents a single text element
/// within a text vector command. It contains the name of the text element
/// and its value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "oneText")]
pub struct OneText {
    /// Name of this text element
    #[serde(rename = "@name")]
    pub name: String,
    /// Text value
    #[serde(rename = "$text")]
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_text() {
        let text = OneText {
            name: "text1".to_string(),
            value: "test value".to_string(),
        };

        assert_eq!(text.name, "text1");
        assert_eq!(text.value, "test value");
    }
}
