mod concrete {
    use std::fs;

    mod pa {
        pub use super::super::super::*;
    }

    #[test]
    fn t_parse_default_sink() {
        assert_eq!(None, pa::pactl_info_find_default_sink(""));
        assert_eq!(
            None,
            pa::pactl_info_find_default_sink("Mumbo Jumbo: stuff")
        );
        assert_eq!(None, pa::pactl_info_find_default_sink("Default Sink:"));
        assert_eq!(None, pa::pactl_info_find_default_sink("Default Sink: "));
        assert_eq!(
            Some("foo"),
            pa::pactl_info_find_default_sink("Default Sink: foo")
        );
        assert_eq!(
            None,
            pa::pactl_info_find_default_sink("Default Sink: foo bar")
        );
        assert_eq!(
            Some("foo.bar_baz-qux"),
            pa::pactl_info_find_default_sink("Default Sink: foo.bar_baz-qux")
        );
        assert_eq!(
            Some("alsa_output.pci-0000_00_1f.3.analog-stereo"),
            pa::pactl_info_find_default_sink(
                &fs::read_to_string("tests/pactl-info.txt").unwrap()
            )
        );
    }

    #[test]
    fn t_vol_str_parse() {
        assert_eq!(None, pa::vol_str_parse(""));
        assert_eq!(None, pa::vol_str_parse("%"));
        assert_eq!(None, pa::vol_str_parse("foo%"));
        assert_eq!(None, pa::vol_str_parse("%foo"));
        assert_eq!(None, pa::vol_str_parse("%5"));
        assert_eq!(Some(5), pa::vol_str_parse("5%"));
        assert_eq!(Some(5), pa::vol_str_parse("05%"));
        assert_eq!(Some(5), pa::vol_str_parse("005%"));
        assert_eq!(Some(50), pa::vol_str_parse("50%"));
        assert_eq!(Some(100), pa::vol_str_parse("100%"));
    }

    #[test]
    fn t_seq_parse() {
        assert_eq!(None, pa::seq_parse(""));
        assert_eq!(None, pa::seq_parse("foo"));
        assert_eq!(None, pa::seq_parse("#foo"));
        assert_eq!(None, pa::seq_parse("foo#"));
        assert_eq!(None, pa::seq_parse("#"));
        assert_eq!(Some(5), pa::seq_parse("#5"));
        assert_eq!(Some(5), pa::seq_parse("#05"));
        assert_eq!(Some(5), pa::seq_parse("#005"));
        assert_eq!(Some(50), pa::seq_parse("#50"));
        assert_eq!(Some(100), pa::seq_parse("#100"));
    }

    #[test]
    fn t_pactl_list_sinks_parse() {
        let given = fs::read_to_string("tests/pactl-list-sinks.txt").unwrap();
        let expected = vec![pa::Sink {
            _seq: 0,
            name: "alsa_output.pci-0000_00_1f.3.analog-stereo",
            mute: false,
            vol_left: 50,
            vol_right: 50,
        }];
        let actual = pa::pactl_list_sinks_parse(&given).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn t_pactl_list_source_outputs_parse() {
        let given = fs::read_to_string("tests/pactl-list-source-outputs.txt")
            .unwrap();
        let expected = vec![65];
        let actual = pa::pactl_list_source_outputs_parse(&given).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn t_update_parse() {
        // line := "Event " event " on " stream " " seq
        // event := "'new'" | "'change'" | "'remove'"
        // stream := "sink" | "source-output"
        // seq := "#"[0-9]+
        assert_eq!(
            (pa::Event::New, pa::Stream::Sink, 1),
            pa::update_parse("Event 'new' on sink #1").unwrap().unwrap()
        );
        assert_eq!(
            (pa::Event::Change, pa::Stream::Sink, 2),
            pa::update_parse("Event 'change' on sink #2")
                .unwrap()
                .unwrap()
        );
        assert_eq!(
            (pa::Event::Remove, pa::Stream::Sink, 3),
            pa::update_parse("Event 'remove' on sink #3")
                .unwrap()
                .unwrap()
        );
        assert_eq!(
            (pa::Event::New, pa::Stream::SourceOutput, 4),
            pa::update_parse("Event 'new' on source-output #4")
                .unwrap()
                .unwrap()
        );
        assert!(pa::update_parse("Event 'poop' on sink #3").is_none());
        assert!(pa::update_parse("Events 'new' on sink #3").is_none());
        assert!(pa::update_parse("Event 'new' on sink 3").is_none());
        assert!(pa::update_parse("Event 'new' on toilet #3").is_none());
    }
}
