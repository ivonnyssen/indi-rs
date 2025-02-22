use super::*;
use std::str::FromStr;
use crate::prelude::PropertyPerm;
use crate::property::{PropertyState, SwitchRule, SwitchState};
use crate::message::basic::BLOBEnable;

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
            assert_eq!(v.label, Some("Slew Rate".to_string()));
            assert_eq!(v.group, Some("Motion".to_string()));
            assert_eq!(v.state, PropertyState::Ok);
            assert_eq!(v.perm, PropertyPerm::Rw);
            assert_eq!(v.rule, SwitchRule::OneOfMany);
            assert_eq!(v.timeout, Some(60));
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
    let xml = r#"<enableBLOB device="CCD" name="CCD1">Never</enableBLOB>"#;
    let parsed: MessageType = xml.parse().unwrap();
    match parsed {
        MessageType::EnableBLOB(v) => {
            assert_eq!(v.device, "CCD");
            assert_eq!(v.name, Some("CCD1".to_string()));
            assert_eq!(v.value, BLOBEnable::Never);
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_enable_blob() {
    let xml = r#"<enableBLOB device="CCD">Never</enableBLOB>"#;
    let parsed: MessageType = xml.parse().unwrap();
    match parsed {
        MessageType::EnableBLOB(v) => {
            assert_eq!(v.device, "CCD");
            assert_eq!(v.name, None);
            assert_eq!(v.value, BLOBEnable::Never);
        }
        _ => panic!("Wrong message type"),
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
            assert_eq!(v.numbers.len(), 2);
            assert_eq!(v.numbers[0].name, "RA");
            assert_eq!(v.numbers[0].value, "12.345678");
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
            assert_eq!(v.switches.len(), 4);
            assert_eq!(v.switches[0].name, "SLEW_GUIDE");
            assert_eq!(v.switches[0].value, SwitchState::Off);
        }
        _ => panic!("Expected SetSwitchVector variant"),
    }
}

#[test]
fn test_actual_get_properties_message() {
    // Test the actual getProperties message from the server log
    let msg = r#"<getProperties version="1.7" />"#;
    let parsed: MessageType = msg.parse().unwrap();
    match parsed {
        MessageType::GetProperties(props) => {
            assert_eq!(props.version, "1.7");
            assert!(props.device.is_none());
            assert!(props.name.is_none());
        }
        _ => panic!("Expected GetProperties message"),
    }
}

#[test]
fn test_actual_driver_info_response() {
    // Test the actual DRIVER_INFO response from the server log
    let msg = r#"<defTextVector device="QHY CCD QHY5III290C-1ca" name="DRIVER_INFO" label="Driver Info" group="General Info" state="Idle" perm="ro" timeout="60" timestamp="2025-02-21T22:05:32">
    <defText name="DRIVER_NAME" label="Name">
QHY CCD
    </defText>
    <defText name="DRIVER_EXEC" label="Exec">
indi_qhy_ccd
    </defText>
    <defText name="DRIVER_VERSION" label="Version">
2.8
    </defText>
    <defText name="DRIVER_INTERFACE" label="Interface">
2
    </defText>
</defTextVector>"#;
    
    let parsed: MessageType = msg.parse().unwrap();
    match parsed {
        MessageType::DefTextVector(vector) => {
            assert_eq!(vector.device, "QHY CCD QHY5III290C-1ca");
            assert_eq!(vector.name, "DRIVER_INFO");
            assert_eq!(vector.label, "Driver Info");
            assert_eq!(vector.group, "General Info");
            assert_eq!(vector.state, PropertyState::Idle);
            assert_eq!(vector.perm, PropertyPerm::Ro);
            assert_eq!(vector.timeout, 60);
            assert_eq!(vector.timestamp, "2025-02-22T02:50:35");
            
            assert_eq!(vector.texts.len(), 4);
            assert_eq!(vector.texts[0].name, "DRIVER_NAME");
            assert_eq!(vector.texts[0].label, "Name");
            assert_eq!(vector.texts[0].value, "QHY CCD");
        }
        _ => panic!("Expected DefTextVector message"),
    }
}

#[test]
fn test_actual_debug_switch_response() {
    // Test the actual DEBUG switch response from the server log
    let msg = r#"<defSwitchVector device="QHY CCD QHY5III290C-1ca" name="CONNECTION" label="Connection" group="Main Control" state="Idle" perm="rw" rule="OneOfMany" timeout="60" timestamp="2025-02-21T22:05:32">
    <defSwitch name="CONNECT" label="Connect">
Off
    </defSwitch>
    <defSwitch name="DISCONNECT" label="Disconnect">
On
    </defSwitch>
</defSwitchVector>"#;

    let parsed: MessageType = msg.parse().unwrap();
    match parsed {
        MessageType::DefSwitchVector(vector) => {
            assert_eq!(vector.device, "QHY CCD QHY5III290C-1ca");
            assert_eq!(vector.name, "CONNECTION");
            assert_eq!(vector.label, Some("Connection".to_string()));
            assert_eq!(vector.group, Some("Main Control".to_string()));
            assert_eq!(vector.state, PropertyState::Idle);
            assert_eq!(vector.perm, PropertyPerm::Rw);
            assert_eq!(vector.rule, SwitchRule::OneOfMany);
            assert_eq!(vector.timeout, Some(60));
            assert_eq!(vector.timestamp, "2025-02-21T22:05:32");

            assert_eq!(vector.switches.len(), 2);
            assert_eq!(vector.switches[0].name, "CONNECT");
            assert_eq!(vector.switches[0].label, "Connect");
            assert_eq!(vector.switches[0].state, SwitchState::Off);
            assert_eq!(vector.switches[1].name, "DISCONNECT");
            assert_eq!(vector.switches[1].label, "Disconnect");
            assert_eq!(vector.switches[1].state, SwitchState::On);
        }
        _ => panic!("Expected DefSwitchVector message"),
    }
}

#[test]
fn test_blob_message_serialization() {
    use crate::message::blob::{OneBLOB, SetBLOBVector};
    use crate::property::PropertyState;

    let test_data = vec![1, 2, 3, 4, 5];
    let blob = OneBLOB::new(
        "test_blob".to_string(),
        "application/octet-stream".to_string(),
        test_data.clone(),
    );

    let blob_vector = SetBLOBVector {
        device: "TestDevice".to_string(),
        name: "TestBLOB".to_string(),
        state: PropertyState::Ok,
        timestamp: "2024-02-21T19:30:00".to_string(),
        blobs: vec![blob],
    };

    let xml = quick_xml::se::to_string(&blob_vector).unwrap();
    assert!(xml.contains("setBLOBVector"));
    assert!(xml.contains("device=\"TestDevice\""));
    assert!(xml.contains("name=\"TestBLOB\""));
    assert!(xml.contains("state=\"Ok\""));
    assert!(xml.contains("oneBLOB"));
    assert!(xml.contains("format=\"application/octet-stream\""));

    let decoded: SetBLOBVector = quick_xml::de::from_str(&xml).unwrap();
    assert_eq!(decoded.device, "TestDevice");
    assert_eq!(decoded.blobs[0].get_data().unwrap(), test_data);
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;
    use quick_xml::se::to_string;

    #[test]
    fn test_enable_blob() {
        let xml = r#"<enableBLOB device="CCD">Never</enableBLOB>"#;
        let parsed: MessageType = xml.parse().unwrap();
        match parsed {
            MessageType::EnableBLOB(v) => {
                assert_eq!(v.device, "CCD");
                assert_eq!(v.name, None);
                assert_eq!(v.value, BLOBEnable::Never);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_number_property_formatting() {
        use crate::message::definition::DefNumber;

        // Test regular floating point format
        let mut num = DefNumber::new(
            "test".to_string(),
            "Test Number".to_string(),
            "%.2f".to_string(),
            -10.0,
            10.0,
            0.1,
            5.25,
        ).unwrap();

        assert_eq!(num.get_value().unwrap(), 5.25);
        assert_eq!(num.get_min().unwrap(), -10.0);
        assert_eq!(num.get_max().unwrap(), 10.0);
        assert_eq!(num.get_step().unwrap(), 0.1);

        // Test sexagesimal format (HH:MM:SS)
        let mut num = DefNumber::new(
            "ra".to_string(),
            "Right Ascension".to_string(),
            "%10.6m".to_string(),
            0.0,
            24.0,
            0.0,
            12.5,  // 12:30:00
        ).unwrap();

        assert_eq!(num.get_value().unwrap(), 12.5);
        assert_eq!(num.value, " 12:30:00");

        // Test value validation
        assert!(num.validate_value(12.0).is_ok());
        assert!(num.validate_value(-1.0).is_err());
        assert!(num.validate_value(25.0).is_err());

        // Test value setting
        num.set_value(6.25).unwrap();  // Should format as 06:15:00
        assert_eq!(num.value, " 06:15:00");
        assert_eq!(num.get_value().unwrap(), 6.25);
    }

    #[test]
    fn test_number_vector_serialization() {
        use crate::message::definition::{DefNumber, DefNumberVector};
        use crate::property::{PropertyState, PropertyPerm};

        let number = DefNumber::new(
            "ra".to_string(),
            "Right Ascension".to_string(),
            "%10.6m".to_string(),
            0.0,
            24.0,
            0.0,
            12.5,
        ).unwrap();

        let vector = DefNumberVector {
            device: "Telescope".to_string(),
            name: "EQUATORIAL_EOD_COORD".to_string(),
            label: "RA/DEC".to_string(),
            group: "Main Control".to_string(),
            state: PropertyState::Ok,
            perm: PropertyPerm::Rw,
            timeout: 0,
            timestamp: "2024-02-21T19:30:00".to_string(),
            numbers: vec![number],
        };

        let xml = to_string(&vector).unwrap();
        assert!(xml.contains("defNumberVector"));
        assert!(xml.contains("device=\"Telescope\""));
        assert!(xml.contains("format=\"%10.6m\""));
        assert!(xml.contains("state=\"Ok\""));
        assert!(xml.contains(" 12:30:00"));
    }
}
