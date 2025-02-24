pub mod define;
pub mod new;
pub mod one;
pub mod rule;
pub mod set;
pub mod state;

pub use define::{DefSwitch, DefSwitchVector};
pub use new::NewSwitchVector;
pub use one::OneSwitch;
pub use rule::SwitchRule;
pub use set::SetSwitchVector;
pub use state::SwitchState;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::switch::SwitchRule;
    use crate::prelude::{PropertyPerm, PropertyState};

    #[test]
    fn test_switch_vector_optional_fields() {
        let vector = DefSwitchVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: None,
            group: None,
            state: PropertyState::Idle,
            perm: PropertyPerm::Rw,
            rule: SwitchRule::OneOfMany,
            timeout: None,
            timestamp: None,
            message: None,
            switches: vec![],
        };

        assert!(vector.label.is_none());
        assert!(vector.group.is_none());
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
    }
}
