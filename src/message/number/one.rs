use serde::{Deserialize, Serialize};

/// One number element used in number vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneNumber {
    /// Number name
    #[serde(rename = "@name")]
    pub name: String,
    /// Number value
    #[serde(rename = "$text")]
    pub value: f64,
}
