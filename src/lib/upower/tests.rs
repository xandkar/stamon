use std::iter::zip;

use super::*;

// TODO Multi-battery tests.
// TODO Examine state in tests.

#[test]
fn dump() {
    let output: String =
        std::fs::read_to_string("tests/upower-dump.txt").unwrap();
    let mut lines = output.lines().map(|l| l.to_string());
    let messages_produced: Vec<Msg> =
        Messages::from_output_lines(&mut lines).collect();
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
    let states_produced: Vec<StateAggregate> =
        StateAggregates::from_messages(&mut messages_produced.into_iter())
            .collect();
    let states_expected = vec![
        (Direction::Decreasing, std::f32::NAN),
        (Direction::Decreasing, 97.9156),
        (Direction::Decreasing, 97.9156),
    ];

    // State aggregates cannot be compared directly, because they contain
    // floats and we do expect them to at least initially be NaN.
    for ((dir_expected, pct_expected), (dir_produced, pct_produced)) in
        zip(states_expected, states_produced)
    {
        assert_eq!(dir_expected, dir_produced);
        assert!(matches!(
            pct_expected.partial_cmp(&pct_produced),
            None | Some(std::cmp::Ordering::Equal),
        ));
    }
}

#[test]
fn monitor() {
    let output: String =
        std::fs::read_to_string("tests/upower-monitor-detail.txt").unwrap();
    let mut lines = output.lines().map(|l| l.to_string());
    let messages_produced: Vec<Msg> =
        Messages::from_output_lines(&mut lines).collect();
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
