mod define;
mod new;
mod one;
mod set;
mod enable;

pub use define::*;
pub use new::*;
pub use one::*;
pub use set::SetBLOBVector;
pub use enable::{EnableBlob, BLOBEnable};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::common::PropertyState;
    use crate::prelude::PropertyPerm;

    #[test]
    fn test_blob_vector_optional_fields() {
        let vector = DefBLOBVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: None,
            group: None,
            state: PropertyState::Idle,
            perm: PropertyPerm::Rw,
            timeout: None,
            timestamp: None,
            message: None,
            blobs: vec![],
        };

        assert!(vector.label.is_none());
        assert!(vector.group.is_none());
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
    }
}
