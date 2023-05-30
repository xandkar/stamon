use super::{msg, state};

// TODO Multi-battery tests.
// TODO Examine state in tests.

#[test]
fn dump() {
    let output: String =
        std::fs::read_to_string("tests/upower-dump.txt").unwrap();
    let lines = output.lines().map(|l| l.to_string());
    let messages_produced: Vec<msg::Msg> =
        msg::Messages::from_lines(Box::new(lines)).collect();
    let messages_expected: Vec<msg::Msg> = vec![
        msg::Msg::LinePower(msg::LinePower {
            path: "AC".to_string(),
            online: false,
        }),
        msg::Msg::Battery(msg::Battery {
            path: "BAT0".to_string(),
            state: msg::BatteryState::Discharging,
            energy: 87.2898,
            energy_full: 89.148,
        }),
        msg::Msg::Battery(msg::Battery {
            path: "/org/freedesktop/UPower/devices/DisplayDevice".to_string(),
            state: msg::BatteryState::Discharging,
            energy: 87.2898,
            energy_full: 89.148,
        }),
    ];
    assert_eq!(&messages_expected, &messages_produced);

    let mut state = state::State::new("u ", &[]).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    for msg in messages_produced {
        {
            use crate::pipeline::State;
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
    let messages_produced: Vec<msg::Msg> =
        msg::Messages::from_lines(Box::new(lines)).collect();
    dbg!(&messages_produced);
    let messages_expected: Vec<msg::Msg> = vec![
        msg::Msg::Battery(msg::Battery {
            path: "BAT0".to_string(),
            state: msg::BatteryState::Discharging,
            energy: 42.8868,
            energy_full: 89.148,
        }),
        msg::Msg::Battery(msg::Battery {
            path: "BAT0".to_string(),
            state: msg::BatteryState::Discharging,
            energy: 42.8868,
            energy_full: 89.148,
        }),
        msg::Msg::Battery(msg::Battery {
            path: "BAT0".to_string(),
            state: msg::BatteryState::Discharging,
            energy: 42.8868,
            energy_full: 89.148,
        }),
        msg::Msg::Battery(msg::Battery {
            path: "BAT0".to_string(),
            state: msg::BatteryState::Discharging,
            energy: 42.8868,
            energy_full: 89.148,
        }),
        msg::Msg::LinePower(msg::LinePower {
            path: "AC".to_string(),
            online: false,
        }),
        msg::Msg::LinePower(msg::LinePower {
            path: "AC".to_string(),
            online: false,
        }),
    ];
    assert_eq!(&messages_expected, &messages_produced);
}
