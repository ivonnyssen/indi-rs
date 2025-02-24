mod define;

pub use define::{DefLight, DefLightVector};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::common::PropertyState;

    #[test]
    fn test_light_vector_optional_fields() {
        let vector = DefLightVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: None,
            group: None,
            state: PropertyState::Idle,
            timestamp: None,
            message: None,
            lights: vec![],
        };

        assert!(vector.label.is_none());
        assert!(vector.group.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
    }
}
