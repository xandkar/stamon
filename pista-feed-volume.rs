use std::io::{BufRead, BufReader};

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(long = "prefix", default_value = "v ")]
    prefix: String,

    #[clap(long = "postfix", default_value = "")]
    postfix: String,
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();
    let cli = Cli::parse();
    let pre = cli.prefix.as_str();
    let post = cli.postfix.as_str();

    Volume::fetch_and_print(pre, post);

    let mut cmd = std::process::Command::new("pactl")
        .arg("subscribe")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdout = cmd.stdout.as_mut().unwrap();
        for line_result in BufReader::new(stdout).lines() {
            match line_result {
                Err(e) => {
                    log::error!("Failure to read output line from 'pactl, subscribe': {:?}", e)
                }
                Ok(line) => {
                    if line.starts_with("Event 'change' on sink") {
                        // TODO Should we bother distinguishing which sink to react to?
                        //      Maybe not, because sink indices could change.
                        Volume::fetch_and_print(pre, post)
                    }
                }
            }
        }
    }
    cmd.wait().unwrap();
}

#[derive(PartialEq, Eq, Debug)]
enum Volume {
    Muted,
    Volume(u64, u64),
}

impl Volume {
    pub fn fetch_and_print(prefix: &str, postfix: &str) {
        match Self::fetch() {
            Ok(Self::Muted) => {
                println!("{}X{}", prefix, postfix)
            }
            Ok(Self::Volume(left, right)) => {
                // TODO CLI option to aggregate or pick one.
                let avg = (left + right) / 2;
                println!("{}{:3}%{}", prefix, avg, postfix)
            }
            Err(e) => {
                log::error!("{:?}", e)
            }
        }
    }

    fn fetch() -> Result<Self> {
        // Default sink could change, so need to look it up every time.
        let pactl_info = cmd("pactl", &["info"])?;
        let sink = pactl_info_to_default_sink(std::str::from_utf8(
            &pactl_info.stdout,
        )?)
        .ok_or_else(|| anyhow!("Failure to get default sink."))?;
        let out = cmd("pactl", &["list", "sinks"])?;
        let data = std::str::from_utf8(&out.stdout)?;
        let volume = pactl_list_sinks_to_volume(data, sink)?;
        Ok(volume)
    }
}

fn cmd(cmd: &str, args: &[&str]) -> Result<std::process::Output> {
    let out = std::process::Command::new(cmd).args(args).output()?;
    if out.status.success() {
        Ok(out)
    } else {
        let err_msg =
            format!("Failure in '{} {:?}'. out: {:?}", cmd, args, out);
        log::error!("{}", err_msg);
        Err(anyhow!(err_msg))
    }
}

fn pactl_info_to_default_sink(data: &str) -> Option<&str> {
    for line in data.lines() {
        if line.starts_with("Default Sink:") {
            return line.split_whitespace().nth(2);
        }
    }
    None
}

fn pactl_list_sinks_to_volume(data: &str, sink: &str) -> Result<Volume> {
    let mut name: Option<&str> = None;
    let mut mute: Option<&str> = None;
    for line in data.lines() {
        match () {
            _ if line.starts_with("Sink #") => {
                name = None;
                mute = None;
            }
            _ if line.starts_with("	Name:") => {
                name = line.split_whitespace().nth(1);
            }
            _ if line.starts_with("	Mute:") => {
                mute = line.split_whitespace().nth(1);
            }
            _ if line.starts_with("	Volume:") => {
                if let ["Volume:", "front-left:", _, "/", l, "/", _, "dB,", "front-right:", _, "/", r, "/", _, "dB"] =
                    line.split_whitespace().collect::<Vec<&str>>()[..]
                {
                    if let (Some(l), Some(r)) =
                        (l.strip_suffix('%'), r.strip_suffix('%'))
                    {
                        if let (Ok(l), Ok(r)) =
                            (l.parse::<u64>(), r.parse::<u64>())
                        {
                            match name {
                                Some(name) if name == sink => match mute {
                                    None => (),
                                    Some("yes") => return Ok(Volume::Muted),
                                    Some("no") => {
                                        return Ok(Volume::Volume(l, r))
                                    }
                                    Some(invalid) => {
                                        return Err(anyhow!(
                                            "Invalid mute value: {:?}",
                                            invalid
                                        ))
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
    Err(anyhow!("Target sink not found"))
}

#[test]
fn test_parse_default_sink() {
    assert_eq!(None, pactl_info_to_default_sink(&""));
    assert_eq!(None, pactl_info_to_default_sink(&"Mumbo Jumbo: stuff"));
    assert_eq!(None, pactl_info_to_default_sink(&"Default Sink:"));
    assert_eq!(None, pactl_info_to_default_sink(&"Default Sink: "));
    assert_eq!(
        Some("foo"),
        pactl_info_to_default_sink(&"Default Sink: foo")
    );
    assert_eq!(
        Some("foo"),
        pactl_info_to_default_sink(&"Default Sink: foo bar")
    );
    assert_eq!(
        Some("foo.bar_baz-qux"),
        pactl_info_to_default_sink(&"Default Sink: foo.bar_baz-qux")
    );
    assert_eq!(
        Some("alsa_output.pci-0000_00_1f.3.analog-stereo"),
        pactl_info_to_default_sink(
            &std::fs::read_to_string("tests/pactl-info.txt").unwrap()
        )
    );
}

#[test]
fn test_pactl_list_sinks_to_volume() {
    assert_eq!(
        Some(Volume::Volume(50, 50)),
        pactl_list_sinks_to_volume(
            &std::fs::read_to_string("tests/pactl-list-sinks.txt").unwrap(),
            "alsa_output.pci-0000_00_1f.3.analog-stereo"
        )
        .ok()
    )
}
