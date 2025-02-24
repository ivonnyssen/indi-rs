pub mod define;
pub mod new;
pub mod one;
pub mod set;

pub use define::{DefText, DefTextVector};
pub use new::NewTextVector;
pub use one::OneText;
pub use set::SetTextVector;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::PropertyState;
    use crate::prelude::PropertyPerm;

    #[test]
    fn test_text_vector_optional_fields() {
        let vector = DefTextVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: None,
            group: None,
            state: PropertyState::Idle,
            perm: PropertyPerm::Rw,
            timeout: None,
            timestamp: None,
            message: None,
            texts: vec![],
        };

        assert!(vector.label.is_none());
        assert!(vector.group.is_none());
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
    }
}
