// TODO Replace * with adhoc modules.
use super::*;

// TODO Multi-battery tests.
// TODO Examine state in tests.

#[test]
fn dump() {
    let output: String =
        std::fs::read_to_string("tests/upower-dump.txt").unwrap();
    let lines = output.lines().map(|l| l.to_string());
    let messages_produced: Vec<Msg> =
        Messages::from_output_lines(Box::new(lines)).collect();
    let messages_expected: Vec<Msg> = vec![
        Msg::LinePower(LinePower {
            path: "AC".to_string(),
            online: false,
        }),
        Msg::Battery(Battery {
            path: "BAT0".to_string(),
            state: BatteryState::Discharging,
            energy: 87.2898,
            energy_full: 89.148,
        }),
        Msg::Battery(Battery {
            path: "/org/freedesktop/UPower/devices/DisplayDevice".to_string(),
            state: BatteryState::Discharging,
            energy: 87.2898,
            energy_full: 89.148,
        }),
    ];
    assert_eq!(&messages_expected, &messages_produced);

    let mut state = State::new("u ", &[]).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    for msg in messages_produced {
        {
            use crate::State;
            state.update(msg).unwrap();
            state.display(&mut buf).unwrap();
        }
    }
    assert_eq!(
        vec!["u <---%", "u < 97%", "u < 97%"],
        String::from_utf8(buf)
            .unwrap()
            .lines()
            .collect::<Vec<&str>>()
    );
}

#[test]
fn monitor() {
    let output: String =
        std::fs::read_to_string("tests/upower-monitor-detail.txt").unwrap();
    let lines = output.lines().map(|l| l.to_string());
    let messages_produced: Vec<Msg> =
        Messages::from_output_lines(Box::new(lines)).collect();
    dbg!(&messages_produced);
    let messages_expected: Vec<Msg> = vec![
        Msg::Battery(Battery {
            path: "BAT0".to_string(),
            state: BatteryState::Discharging,
            energy: 42.8868,
            energy_full: 89.148,
        }),
        Msg::Battery(Battery {
            path: "BAT0".to_string(),
            state: BatteryState::Discharging,
            energy: 42.8868,
            energy_full: 89.148,
        }),
        Msg::Battery(Battery {
            path: "BAT0".to_string(),
            state: BatteryState::Discharging,
            energy: 42.8868,
            energy_full: 89.148,
        }),
        Msg::Battery(Battery {
            path: "BAT0".to_string(),
            state: BatteryState::Discharging,
            energy: 42.8868,
            energy_full: 89.148,
        }),
        Msg::LinePower(LinePower {
            path: "AC".to_string(),
            online: false,
        }),
        Msg::LinePower(LinePower {
            path: "AC".to_string(),
            online: false,
        }),
    ];
    assert_eq!(&messages_expected, &messages_produced);
}
