use std::fs;

use super::*;

#[test]
fn t_parse_default_sink() {
    assert_eq!(None, pactl_info_find_default_sink(""));
    assert_eq!(None, pactl_info_find_default_sink("Mumbo Jumbo: stuff"));
    assert_eq!(None, pactl_info_find_default_sink("Default Sink:"));
    assert_eq!(None, pactl_info_find_default_sink("Default Sink: "));
    assert_eq!(
        Some("foo"),
        pactl_info_find_default_sink("Default Sink: foo")
    );
    assert_eq!(None, pactl_info_find_default_sink("Default Sink: foo bar"));
    assert_eq!(
        Some("foo.bar_baz-qux"),
        pactl_info_find_default_sink("Default Sink: foo.bar_baz-qux")
    );
    assert_eq!(
        Some("alsa_output.pci-0000_00_1f.3.analog-stereo"),
        pactl_info_find_default_sink(
            &fs::read_to_string("tests/pactl-info.txt").unwrap()
        )
    );
}

#[test]
fn t_vol_str_parse() {
    assert_eq!(None, vol_str_parse(""));
    assert_eq!(None, vol_str_parse("%"));
    assert_eq!(None, vol_str_parse("foo%"));
    assert_eq!(None, vol_str_parse("%foo"));
    assert_eq!(None, vol_str_parse("%5"));
    assert_eq!(Some(5), vol_str_parse("5%"));
    assert_eq!(Some(5), vol_str_parse("05%"));
    assert_eq!(Some(5), vol_str_parse("005%"));
    assert_eq!(Some(50), vol_str_parse("50%"));
    assert_eq!(Some(100), vol_str_parse("100%"));
}

#[test]
fn t_seq_parse() {
    assert_eq!(None, seq_parse(""));
    assert_eq!(None, seq_parse("foo"));
    assert_eq!(None, seq_parse("#foo"));
    assert_eq!(None, seq_parse("foo#"));
    assert_eq!(None, seq_parse("#"));
    assert_eq!(Some(5), seq_parse("#5"));
    assert_eq!(Some(5), seq_parse("#05"));
    assert_eq!(Some(5), seq_parse("#005"));
    assert_eq!(Some(50), seq_parse("#50"));
    assert_eq!(Some(100), seq_parse("#100"));
}

#[test]
fn t_pactl_list_sinks_parse() {
    let given = fs::read_to_string("tests/pactl-list-sinks.txt").unwrap();
    let expected = vec![Sink {
        _seq: 0,
        name: "alsa_output.pci-0000_00_1f.3.analog-stereo",
        mute: false,
        vol_left: 50,
        vol_right: 50,
    }];
    let actual = pactl_list_sinks_parse(&given).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn t_pactl_list_source_outputs_parse() {
    let given =
        fs::read_to_string("tests/pactl-list-source-outputs.txt").unwrap();
    let expected = vec![65];
    let actual = pactl_list_source_outputs_parse(&given).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn t_update_parse() {
    // line := "Event " event " on " stream " " seq
    // event := "'new'" | "'change'" | "'remove'"
    // stream := "sink" | "source-output"
    // seq := "#"[0-9]+
    assert_eq!(
        (Event::New, Stream::Sink, 1),
        update_parse("Event 'new' on sink #1").unwrap().unwrap()
    );
    assert_eq!(
        (Event::Change, Stream::Sink, 2),
        update_parse("Event 'change' on sink #2").unwrap().unwrap()
    );
    assert_eq!(
        (Event::Remove, Stream::Sink, 3),
        update_parse("Event 'remove' on sink #3").unwrap().unwrap()
    );
    assert_eq!(
        (Event::New, Stream::SourceOutput, 4),
        update_parse("Event 'new' on source-output #4")
            .unwrap()
            .unwrap()
    );
    assert!(update_parse("Event 'poop' on sink #3").is_none());
    assert!(update_parse("Events 'new' on sink #3").is_none());
    assert!(update_parse("Event 'new' on sink 3").is_none());
    assert!(update_parse("Event 'new' on toilet #3").is_none());
}
