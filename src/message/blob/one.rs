use serde::{Deserialize, Serialize};

/// One BLOB element that can be used in both setBLOBVector and newBLOBVector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "oneBLOB")]
pub struct OneBLOB {
    /// BLOB name
    #[serde(rename = "@name")]
    pub name: String,
    /// BLOB size
    #[serde(rename = "@size")]
    pub size: usize,
    /// BLOB format
    #[serde(rename = "@format")]
    pub format: String,
    /// BLOB data encoded in base64
    #[serde(rename = "$text")]
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_blob() {
        let blob = OneBLOB {
            name: "test_blob".to_string(),
            size: 100,
            format: ".fits".to_string(),
            data: "base64encodeddata".to_string(),
        };

        assert_eq!(blob.name, "test_blob");
        assert_eq!(blob.size, 100);
        assert_eq!(blob.format, ".fits");
        assert_eq!(blob.data, "base64encodeddata");
    }
}
