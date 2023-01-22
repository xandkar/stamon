// TODO Rewrite with pulseaudio bindings.
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

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    let pre = cli.prefix.as_str();
    let post = cli.postfix.as_str();

    Volume::fetch_and_print(pre, post);

    let mut cmd = std::process::Command::new("pactl")
        .arg("subscribe")
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    {
        let stdout = cmd
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow!("Failure to get command's stdout."))?;
        for line_result in BufReader::new(stdout).lines() {
            match line_result {
                Err(e) => {
                    tracing::error!("Failure to read output line from 'pactl, subscribe': {:?}", e);
                }
                Ok(line) => {
                    if line.starts_with("Event 'change' on sink") {
                        // TODO Should we bother distinguishing which sink to react to?
                        //      Maybe not, because sink indices could change.
                        Volume::fetch_and_print(pre, post);
                    }
                }
            }
        }
    }
    cmd.wait()?;
    Ok(())
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
                println!("{} X {}", prefix, postfix);
            }
            Ok(Self::Volume(left, right)) => {
                // TODO CLI option to aggregate or pick one.
                let avg = (left + right) / 2;
                println!("{}{:3}%{}", prefix, avg, postfix);
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                println!("{}ERR{}", prefix, postfix);
            }
        }
    }

    fn fetch() -> Result<Self> {
        // Default sink could change, so need to look it up every time.
        let pactl_info = &cmd("pactl", &["info"])?;
        let pactl_info = std::str::from_utf8(pactl_info)?;
        let sink = pactl_info_to_default_sink(pactl_info)?;
        let pactl_list_sinks = cmd("pactl", &["list", "sinks"])?;
        let pactl_list_sinks = std::str::from_utf8(&pactl_list_sinks)?;
        let volume = pactl_list_sinks_to_volume(pactl_list_sinks, sink)?;
        Ok(volume)
    }
}

fn cmd(cmd: &str, args: &[&str]) -> Result<Vec<u8>> {
    let out = std::process::Command::new(cmd).args(args).output()?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        let err_msg =
            format!("Failure in '{} {:?}'. out: {:?}", cmd, args, out);
        tracing::error!("{}", err_msg);
        Err(anyhow!(err_msg))
    }
}

fn pactl_info_to_default_sink(data: &str) -> Result<&str> {
    let prefix = "Default Sink:";
    for line in data.lines() {
        if line.starts_with(prefix) {
            return line.split_whitespace().nth(2).ok_or_else(|| {
                anyhow!(
                    "{:?} missing right-hand-side value: {:?}",
                    prefix,
                    data
                )
            });
        }
    }
    Err(anyhow!("{:?} not found", prefix))
}

fn vol_str_parse(s: &str) -> Result<u64> {
    s.strip_suffix('%')
        .ok_or_else(|| anyhow!("Volume string missing '%': {:?}", s))
        .and_then(|s| {
            s.parse::<u64>()
                .map_err(|_| anyhow!("Volume string number invalid: {:?}", s))
        })
}

fn pactl_list_sinks_to_volume(data: &str, sink: &str) -> Result<Volume> {
    let mut name: Option<&str> = None;
    let mut mute: Option<bool> = None;
    for line in data.lines() {
        match () {
            _ if line.starts_with("Sink #") => {
                name = None;
                mute = None;
            }
            _ if line.starts_with("	Name:") => {
                name = match line.split_whitespace().nth(1) {
                    Some(name) => Some(name),
                    None => {
                        return Err(anyhow!(
                            "Missing value for Name field in line: {line:?}"
                        ))
                    }
                }
            }
            _ if line.starts_with("	Mute:") => {
                mute = match line.split_whitespace().nth(1) {
                    Some("yes") => Some(true),
                    Some("no") => Some(false),
                    Some(m) => {
                        return Err(anyhow!("Invalid Mute value: {m:?}"))
                    }
                    None => {
                        return Err(anyhow!(
                            "Missing value for Mute field line: {line:?}"
                        ))
                    }
                }
            }
            _ if line.starts_with("	Volume:") => {
                match (name, mute) {
                    (Some(name), Some(is_muted)) if name == sink => {
                        if is_muted {
                            return Ok(Volume::Muted);
                        }
                        match line.split_whitespace().collect::<Vec<&str>>()[..]
                        {
                            ["Volume:", "front-left:", _, "/", left, "/", _, "dB,", "front-right:", _, "/", right, "/", _, "dB"] => {
                                return Ok(Volume::Volume(
                                    vol_str_parse(left)?,
                                    vol_str_parse(right)?,
                                ))
                            }
                            _ => {
                                return Err(anyhow!(
                                    "Invalid Volume value: {line:?}"
                                ))
                            }
                        }
                    }
                    (Some(_), Some(_)) => (), // A sink we don't care about.
                    (Some(_), None) => {
                        tracing::error!(
                            "Invalid format - no Mute before Volume."
                        );
                    }
                    (None, Some(_)) => {
                        tracing::error!(
                            "Invalid format - no Name before Volume."
                        );
                    }
                    (None, None) => tracing::error!(
                        "Invalid format - no Name or Mute before Volume."
                    ),
                }
            }
            _ => (), // A line we don't care about.
        }
    }
    Err(anyhow!("Target sink not found"))
}

#[test]
fn test_parse_default_sink() {
    assert!(pactl_info_to_default_sink(&"").is_err());
    assert!(pactl_info_to_default_sink(&"Mumbo Jumbo: stuff").is_err());
    assert!(pactl_info_to_default_sink(&"Default Sink:").is_err());
    assert!(pactl_info_to_default_sink(&"Default Sink: ").is_err());
    assert_eq!(
        Some("foo"),
        pactl_info_to_default_sink(&"Default Sink: foo").ok()
    );
    assert_eq!(
        Some("foo"),
        pactl_info_to_default_sink(&"Default Sink: foo bar").ok()
    );
    assert_eq!(
        Some("foo.bar_baz-qux"),
        pactl_info_to_default_sink(&"Default Sink: foo.bar_baz-qux").ok()
    );
    assert_eq!(
        Some("alsa_output.pci-0000_00_1f.3.analog-stereo"),
        pactl_info_to_default_sink(
            &std::fs::read_to_string("tests/pactl-info.txt").unwrap()
        )
        .ok()
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
