mod define;
mod set;
mod new;
mod one;

pub use define::{DefNumber, DefNumberVector};
pub use new::NewNumberVector;
pub use set::SetNumberVector;
pub use one::OneNumber;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::common::PropertyState;
    use crate::prelude::PropertyPerm;

    #[test]
    fn test_number_vector_optional_fields() {
        let vector = DefNumberVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: None,
            group: None,
            state: PropertyState::Idle,
            perm: PropertyPerm::Rw,
            timeout: None,
            timestamp: None,
            message: None,
            numbers: vec![],
        };

        assert!(vector.label.is_none());
        assert!(vector.group.is_none());
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
    }
}
