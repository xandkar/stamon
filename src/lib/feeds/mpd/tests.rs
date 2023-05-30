/// Target module being tested
mod t {
    pub use super::super::*;
}

/// .display() tests
mod display {
    /// The whole .display() call
    mod full {
        use std::time::Duration;

        use super::super::util;

        use crate::pipeline::State; // .display()

        #[test]
        fn stop_time_none() {
            let mut buf = Vec::new();
            util::state_with(mpd::status::State::Stop, None, None)
                .display(&mut buf)
                .unwrap();
            assert_eq!("-    --:--  ---\n", util::buf_to_string(buf, 16));
        }

        #[test]
        fn play_streaming() {
            let mut buf = Vec::new();
            util::state_with(
                mpd::status::State::Play,
                None,
                Some(Duration::from_secs(5)),
            )
            .display(&mut buf)
            .unwrap();
            assert_eq!(">    00:05  ~~~\n", util::buf_to_string(buf, 16));
        }

        #[test]
        fn play_time_none() {
            let mut buf = Vec::new();
            util::state_with(mpd::status::State::Play, None, None)
                .display(&mut buf)
                .unwrap();
            assert_eq!(">    --:--  ???\n", util::buf_to_string(buf, 16));
        }

        #[test]
        fn pause_time_none() {
            let mut buf = Vec::new();
            util::state_with(mpd::status::State::Pause, None, None)
                .display(&mut buf)
                .unwrap();
            assert_eq!("=    --:--  ???\n", util::buf_to_string(buf, 16));
        }

        #[test]
        fn pause_time_some_50pct() {
            let mut buf = Vec::new();
            util::state_with(
                mpd::status::State::Pause,
                Some(Duration::from_secs(10)),
                Some(Duration::from_secs(5)),
            )
            .display(&mut buf)
            .unwrap();
            assert_eq!("=    00:05  50%\n", util::buf_to_string(buf, 16));
        }

        #[test]
        fn play_time_some_100pct() {
            let mut buf = Vec::new();
            util::state_with(
                mpd::status::State::Play,
                Some(Duration::from_secs(60 * 60)),
                Some(Duration::from_secs(60 * 60)),
            )
            .display(&mut buf)
            .unwrap();
            assert_eq!("> 01:00:00 100%\n", util::buf_to_string(buf, 16));
        }
    }

    /// .display_time() component tests
    mod time {
        use std::time::Duration;

        use super::super::t;
        use super::super::util;

        #[test]
        fn stop_time_none() {
            let mut buf = Vec::new();
            t::State::new(util::SYM).display_time(&mut buf).unwrap();
            assert_eq!("   --:--", util::buf_to_string(buf, 8));
        }

        #[test]
        fn pause_time_some() {
            let mpd_state = mpd::status::State::Pause;
            let duration = Some(Duration::from_secs(10));
            let elapsed = Some(Duration::from_secs(5));
            let mut buf = Vec::new();
            util::state_with(mpd_state, duration, elapsed)
                .display_time(&mut buf)
                .unwrap();
            assert_eq!("   00:05", util::buf_to_string(buf, 8));
        }

        #[test]
        fn stop_time_some() {
            let mpd_state = mpd::status::State::Stop;
            let duration = Some(Duration::from_secs(10));
            let elapsed = Some(Duration::from_secs(5));
            let mut buf = Vec::new();
            util::state_with(mpd_state, duration, elapsed)
                .display_time(&mut buf)
                .unwrap();
            assert_eq!("   --:--", util::buf_to_string(buf, 8));
        }

        #[test]
        fn play_time_some() {
            let mpd_state = mpd::status::State::Play;
            let duration = Some(Duration::from_secs(60 * 60));
            let elapsed = Some(Duration::from_secs(60 * 60));
            let mut buf = Vec::new();
            util::state_with(mpd_state, duration, elapsed)
                .display_time(&mut buf)
                .unwrap();
            assert_eq!("01:00:00", util::buf_to_string(buf, 8));
        }
    }
}

/// Testing helpers. Can use a better name, alas.
mod util {
    use std::time::Duration;

    use super::t;

    pub const SYM: t::Symbols<'static> = t::Symbols {
        prefix: "",
        postfix: "",
        state_play: ">",
        state_pause: "=",
        state_stop: "-",
        state_off: " ",
        pct_when_stopped: "---",
        pct_when_streaming: "~~~",
        pct_when_off: "   ",
    };

    pub fn state_with<'a>(
        mpd_state: mpd::status::State,
        duration: Option<Duration>,
        elapsed: Option<Duration>,
    ) -> t::State<'a> {
        t::State {
            status: mpd_status_new().map(|s| mpd::status::Status {
                state: mpd_state,
                duration,
                elapsed,
                ..s
            }),
            symbols: SYM,
        }
    }

    pub fn mpd_status_new() -> Option<mpd::status::Status> {
        let status = mpd::status::Status::default();
        assert_eq!(status.state, mpd::status::State::Stop);
        assert!(status.duration.is_none());
        assert!(status.elapsed.is_none());
        Some(status)
    }

    pub fn buf_to_string(buf: Vec<u8>, len: usize) -> String {
        let str = String::from_utf8(buf).unwrap();
        dbg!(&str);
        assert_eq!(len, str.len());
        str
    }
}
