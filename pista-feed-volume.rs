use std::collections::{HashMap, HashSet};
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
                    if line.starts_with("Event 'new' on sink") {
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
        let volume =
            pactl_list_sinks_to_volume(data, sink).ok_or_else(|| {
                anyhow!("Failed to get volume for some reason.")
            })?;
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

fn pactl_list_sinks_to_volume(data: &str, sink: &str) -> Option<Volume> {
    let mut sink_id: Option<u64> = None;
    let mut sink_ids: HashSet<u64> = HashSet::new();
    let mut name: HashMap<u64, &str> = HashMap::new();
    let mut mute: HashMap<u64, &str> = HashMap::new();
    let mut vol_left: HashMap<u64, u64> = HashMap::new();
    let mut vol_right: HashMap<u64, u64> = HashMap::new();
    for line in data.lines() {
        match sink_id {
            _ if line.starts_with("Sink #") => {
                if let Some(id) = line.split('#').nth(1) {
                    if let Ok(id) = id.parse::<u64>() {
                        sink_id = Some(id)
                    }
                }
            }
            Some(id) if line.starts_with("	State:") => {
                sink_ids.insert(id);
            }
            Some(id) if line.starts_with("	Name:") => {
                line.split_whitespace().nth(1).map(|n| name.insert(id, n));
            }
            Some(id) if line.starts_with("	Mute:") => {
                line.split_whitespace().nth(1).map(|m| mute.insert(id, m));
            }
            Some(id) if line.starts_with("	Volume:") => {
                if let ["Volume:", "front-left:", _, "/", l, "/", _, "dB,", "front-right:", _, "/", r, "/", _, "dB"] =
                    line.split_whitespace().collect::<Vec<&str>>()[..]
                {
                    if let (Some(l), Some(r)) =
                        (l.strip_suffix('%'), r.strip_suffix('%'))
                    {
                        if let (Ok(l), Ok(r)) =
                            (l.parse::<u64>(), r.parse::<u64>())
                        {
                            vol_left.insert(id, l);
                            vol_right.insert(id, r);
                        }
                    }
                }
            }
            _ => (),
        }
    }
    for id in sink_ids.iter() {
        match (
            name.get(id),
            mute.get(id).copied(),
            vol_left.get(id),
            vol_right.get(id),
        ) {
            (Some(name), Some("yes"), Some(_), Some(_)) if *name == sink => {
                return Some(Volume::Muted)
            }
            (Some(name), Some("no"), Some(left), Some(right))
                if *name == sink =>
            {
                return Some(Volume::Volume(*left, *right))
            }
            _ => (),
        }
    }
    None
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
    )
}
