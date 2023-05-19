use super::*;

#[test]
fn test_parse_default_sink() {
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
            &std::fs::read_to_string("tests/pactl-info.txt").unwrap()
        )
    );
}
