use super::*;
use std::str::FromStr;
use crate::property::{PropertyState, SwitchRule, SwitchState};

#[test]
fn test_parse_message() {
    let xml = r#"<message>Hello World</message>"#;
    let message = MessageType::from_str(xml).unwrap();
    match message {
        MessageType::Message(m) => assert_eq!(m.content, "Hello World"),
        _ => panic!("Expected Message variant"),
    }
}

#[test]
fn test_parse_def_switch_vector() {
    let xml = r#"<defSwitchVector device="Telescope Mount" name="TELESCOPE_SLEW_RATE" label="Slew Rate" group="Motion" state="Ok" perm="rw" rule="OneOfMany" timeout="60" timestamp="2024-01-01T00:00:00">
        <defSwitch name="SLEW_GUIDE" label="Guide">Off</defSwitch>
        <defSwitch name="SLEW_CENTERING" label="Centering">On</defSwitch>
        <defSwitch name="SLEW_FIND" label="Find">Off</defSwitch>
        <defSwitch name="SLEW_MAX" label="Max">Off</defSwitch>
    </defSwitchVector>"#;

    let message = MessageType::from_str(xml).unwrap();
    match message {
        MessageType::DefSwitchVector(v) => {
            assert_eq!(v.device, "Telescope Mount");
            assert_eq!(v.name, "TELESCOPE_SLEW_RATE");
            assert_eq!(v.label, "Slew Rate");
            assert_eq!(v.group, "Motion");
            assert_eq!(v.state, PropertyState::Ok);
            assert_eq!(v.rule, SwitchRule::OneOfMany);
            assert_eq!(v.switches.len(), 4);
            assert_eq!(v.switches[0].name, "SLEW_GUIDE");
            assert_eq!(v.switches[0].state, SwitchState::Off);
        }
        _ => panic!("Expected DefSwitchVector variant"),
    }
}

#[test]
fn test_parse_new_switch_vector() {
    let xml = r#"<newSwitchVector device="Telescope Mount" name="TELESCOPE_SLEW_RATE" timestamp="2024-01-01T00:00:00">
        <oneSwitch name="SLEW_GUIDE">Off</oneSwitch>
        <oneSwitch name="SLEW_CENTERING">On</oneSwitch>
        <oneSwitch name="SLEW_FIND">Off</oneSwitch>
        <oneSwitch name="SLEW_MAX">Off</oneSwitch>
    </newSwitchVector>"#;

    let message = MessageType::from_str(xml).unwrap();
    match message {
        MessageType::NewSwitchVector(v) => {
            assert_eq!(v.device, "Telescope Mount");
            assert_eq!(v.name, "TELESCOPE_SLEW_RATE");
            assert_eq!(v.elements.len(), 4);
            assert_eq!(v.elements[0].name, "SLEW_GUIDE");
            assert_eq!(v.elements[0].value, SwitchState::Off);
        }
        _ => panic!("Expected NewSwitchVector variant"),
    }
}

#[test]
fn test_enable_blob_message() {
    let xml = r#"<enableBLOB device="CCD Simulator">Also</enableBLOB>"#;
    let message = MessageType::from_str(xml).unwrap();
    match message {
        MessageType::EnableBlob(v) => {
            assert_eq!(v.device, "CCD Simulator");
            assert_eq!(v.mode, "Also");
        }
        _ => panic!("Expected EnableBlob variant"),
    }
}

#[test]
fn test_set_number_vector() {
    let xml = r#"<setNumberVector device="Telescope Mount" name="EQUATORIAL_EOD_COORD" timestamp="2024-01-01T00:00:00">
        <oneNumber name="RA">12.345678</oneNumber>
        <oneNumber name="DEC">-45.678901</oneNumber>
    </setNumberVector>"#;

    let message = MessageType::from_str(xml).unwrap();
    match message {
        MessageType::SetNumberVector(v) => {
            assert_eq!(v.device, "Telescope Mount");
            assert_eq!(v.name, "EQUATORIAL_EOD_COORD");
            assert_eq!(v.elements.len(), 2);
            assert_eq!(v.elements[0].name, "RA");
            assert_eq!(v.elements[0].value, "12.345678");
        }
        _ => panic!("Expected SetNumberVector variant"),
    }
}

#[test]
fn test_set_switch_vector() {
    let xml = r#"<setSwitchVector device="Telescope Mount" name="TELESCOPE_SLEW_RATE" timestamp="2024-01-01T00:00:00">
        <oneSwitch name="SLEW_GUIDE">Off</oneSwitch>
        <oneSwitch name="SLEW_CENTERING">On</oneSwitch>
        <oneSwitch name="SLEW_FIND">Off</oneSwitch>
        <oneSwitch name="SLEW_MAX">Off</oneSwitch>
    </setSwitchVector>"#;

    let message = MessageType::from_str(xml).unwrap();
    match message {
        MessageType::SetSwitchVector(v) => {
            assert_eq!(v.device, "Telescope Mount");
            assert_eq!(v.name, "TELESCOPE_SLEW_RATE");
            assert_eq!(v.elements.len(), 4);
            assert_eq!(v.elements[0].name, "SLEW_GUIDE");
            assert_eq!(v.elements[0].value, SwitchState::Off);
        }
        _ => panic!("Expected SetSwitchVector variant"),
    }
}
