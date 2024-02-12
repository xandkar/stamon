// TODO Rewrite with pulseaudio bindings.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Symbols<'a> {
    pub prefix: &'a str,
    pub mic_on: &'a str,
    pub mic_off: &'a str,
    pub mute: &'a str,
    pub equal: &'a str,
    pub approx: &'a str,
}

type Seq = u64;

#[derive(Debug, PartialEq)]
struct Sink<'a> {
    _seq: Seq,
    name: &'a str,
    mute: bool,
    vol_left: u64,
    vol_right: u64,
}

#[derive(Debug, PartialEq)]
enum Event {
    New,
    Change,
    Remove,
}

impl Event {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "'new'" => Some(Self::New),
            "'change'" => Some(Self::Change),
            "'remove'" => Some(Self::Remove),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Stream {
    Sink,
    SourceOutput,
}

impl Stream {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "sink" => Some(Self::Sink),
            "source-output" => Some(Self::SourceOutput),
            _ => None,
        }
    }
}

type Update = (Event, Stream, Seq);

fn updates() -> Result<impl Iterator<Item = Update>> {
    let init_vol_change =
        std::iter::once(Ok((Event::Change, Stream::Sink, 0)));
    let init_source_outputs = source_outputs_list()?;
    let init_source_outputs = init_source_outputs
        .into_iter()
        .map(|seq| Ok((Event::New, Stream::SourceOutput, seq)));
    let updates = init_vol_change
        .chain(init_source_outputs)
        .chain(subscribe()?)
        .filter_map(|result| match result {
            Err(err) => {
                tracing::error!("Failed to read event: {:?}", err);
                None
            }
            Ok(update) => Some(update),
        });
    Ok(updates)
}

enum Volume {
    Muted,
    Exactly(u64),
    Approx(u64),
}

impl Volume {
    fn fetch() -> Result<Self> {
        // Default sink could change, so need to look it up every time.
        let pactl_info = &crate::process::exec("pactl", &["info"])?;
        let pactl_info = std::str::from_utf8(pactl_info)?;
        let target_sink_name = pactl_info_find_default_sink(pactl_info)
            .ok_or_else(|| anyhow!("default sink not found"))?;
        let pactl_list_sinks =
            crate::process::exec("pactl", &["list", "sinks"])?;
        let pactl_list_sinks = std::str::from_utf8(&pactl_list_sinks)?;
        let sinks = pactl_list_sinks_parse(pactl_list_sinks)?;
        let target_sink_opt =
            sinks.iter().find(|s| s.name == target_sink_name);
        let target_vol_opt = target_sink_opt.map(|s| {
            if s.mute {
                Self::Muted
            } else if s.vol_left == s.vol_right {
                Self::Exactly(s.vol_left)
            } else {
                Self::Approx((s.vol_left + s.vol_right) / 2)
            }
        });
        target_vol_opt.ok_or_else(|| anyhow!("Target sink not found"))
    }
}

fn pactl_info_find_default_sink(data: &str) -> Option<&str> {
    for line in data.lines() {
        if let ["Default", "Sink:", name] =
            line.split_whitespace().collect::<Vec<&str>>()[..]
        {
            return Some(name);
        }
    }
    None
}

struct State<'a> {
    symbols: Symbols<'a>,
    mic_sym_len: usize,
    source_outputs: HashSet<Seq>,
    volume: Volume, // TODO Maybe Option instead of init fetch?
}

impl<'a> State<'a> {
    fn new(symbols: Symbols<'a>) -> Result<Self> {
        let source_outputs = HashSet::new();
        let volume = Volume::fetch()?;
        let mic_on = symbols.mic_on.len();
        let mic_off = symbols.mic_off.len();
        Ok(Self {
            symbols,
            mic_sym_len: mic_on.max(mic_off),
            source_outputs,
            volume,
        })
    }
}

impl<'a> crate::pipeline::State for State<'a> {
    type Event = Update;

    fn update(
        &mut self,
        update: Self::Event,
    ) -> Result<Option<Vec<crate::alert::Alert>>> {
        match update {
            (Event::Change, Stream::Sink, _) => {
                self.volume = Volume::fetch()?;
            }
            (Event::New, Stream::SourceOutput, seq) => {
                // TODO Maybe alert here, on mic/source-output additions?
                self.source_outputs.insert(seq);
            }
            (Event::Remove, Stream::SourceOutput, seq) => {
                self.source_outputs.remove(&seq);
            }
            _ => (),
        }
        Ok(None)
    }

    fn display<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        write!(buf, "{}", self.symbols.prefix)?;
        match self.volume {
            Volume::Muted => {
                write!(buf, "{}", self.symbols.mute)?;
            }
            Volume::Exactly(n) => {
                write!(buf, "{}{:3}%", self.symbols.equal, n)?;
            }
            Volume::Approx(n) => {
                write!(buf, "{}{:3}%", self.symbols.approx, n)?;
            }
        }
        let symbol_mic = if self.source_outputs.is_empty() {
            self.symbols.mic_off
        } else {
            self.symbols.mic_on
        };
        writeln!(buf, " {:width$}", symbol_mic, width = self.mic_sym_len)?;
        Ok(())
    }
}

fn subscribe() -> Result<impl Iterator<Item = Result<Update>>> {
    let updates = crate::process::spawn("pactl", &["subscribe"])?.filter_map(
        |line_result| match line_result {
            Ok(line) => update_parse(&line),
            Err(e) => Some(Err(anyhow::Error::from(e))),
        },
    );
    Ok(updates)
}

fn update_parse(line: &str) -> Option<Result<Update>> {
    match line.split_whitespace().collect::<Vec<&str>>()[..] {
        ["Event", event, "on", stream, seq] => {
            match (
                Event::from_str(event),
                Stream::from_str(stream),
                seq_parse(seq),
            ) {
                (Some(event), Some(stream), Some(seq)) => {
                    Some(Ok((event, stream, seq)))
                }
                _ => None,
            }
        }
        _ => {
            tracing::warn!("Unexpected event line: {:?}", line);
            None
        }
    }
}

fn seq_parse(name: &str) -> Option<Seq> {
    name.strip_prefix('#').and_then(|seq| seq.parse().ok())
}

fn pactl_list_sinks_parse<'a>(data: &'a str) -> Result<Vec<Sink<'a>>> {
    let mut sinks: Vec<Sink<'a>> = Vec::new();
    let mut seq: Option<Seq> = None;
    let mut name: Option<&str> = None;
    let mut mute: Option<bool> = None;
    for line in data.lines() {
        let indented = line.starts_with('\t');
        let fields = line.split_whitespace().collect::<Vec<&str>>();
        match fields[..] {
            ["Sink", seq0] if !indented => {
                seq = seq_parse(seq0);
                name = None;
                mute = None;
            }
            ["Name:", name0] if indented => {
                name = Some(name0);
            }
            ["Mute:", "yes"] if indented => {
                mute = Some(true);
            }
            ["Mute:", "no"] if indented => {
                mute = Some(false);
            }
            ["Mute:", other] if indented => {
                return Err(anyhow!("Invalid Mute value: {:?}", other));
            }

            // Volume examples:
            //
            //   Volume: front-left: 9828 /  15% / -49.44 dB,   front-right: 9828 /  15% / -49.44 dB
            //   ["Volume:", "front-left:", "9828", "/", "15%", "/", "-49.44", "dB,", "front-right:", "9828", "/", "15%", "/", "-49.44", "dB"]
            //
            //   Volume: front-left: 30422 /  46%,   front-right: 30422 /  46%
            //   ["Volume:", "front-left:", "30422", "/", "46%,", "front-right:", "30422", "/", "46%"]
            //
            // TODO Maybe handle volume more-generally, like split on ","
            //      and then parse left and right?
            #[rustfmt::skip] // I want these aligned:
            ["Volume:", "front-left:", _, "/", left, "/", _, "dB,", "front-right:", _, "/", right, "/", _, "dB"] |
            ["Volume:", "front-left:", _, "/", left,                "front-right:", _, "/", right,             ]
                if indented =>
            {
                let seq = seq.ok_or_else(|| anyhow!("Missing seq"))?;
                let name = name.ok_or_else(|| anyhow!("Missing name"))?;
                let mute = mute.ok_or_else(|| anyhow!("Missing mute"))?;
                let vol_left = vol_str_parse(left).ok_or_else(|| {
                    anyhow!("Volume string invalid: {:?}", left)
                })?;
                let vol_right = vol_str_parse(right).ok_or_else(|| {
                    anyhow!("Volume string invalid: {:?}", right)
                })?;
                sinks.push(Sink {
                    _seq: seq,
                    name,
                    mute,
                    vol_left,
                    vol_right,
                });
            }
            _ => (),
        }
    }
    Ok(sinks)
}

fn vol_str_parse(s: &str) -> Option<u64> {
    s.trim_end_matches(',')
        .strip_suffix('%')
        .and_then(|s| s.parse().ok())
}

fn source_outputs_list() -> Result<Vec<Seq>> {
    let pactl_list =
        crate::process::exec("pactl", &["list", "source-outputs"])?;
    let pactl_list: &str = std::str::from_utf8(&pactl_list)?;
    Ok(pactl_list_source_outputs_parse(pactl_list))
}

fn pactl_list_source_outputs_parse(data: &str) -> Vec<Seq> {
    let mut sources: HashSet<Seq> = HashSet::new();
    for line in data.lines() {
        if let ["Source", "Output", seq] =
            line.split_whitespace().collect::<Vec<&str>>()[..]
        {
            if let Some(seq) = seq_parse(seq) {
                sources.insert(seq);
            }
        }
    }
    sources.into_iter().collect()
}

pub fn run(symbols: Symbols<'_>) -> Result<()> {
    crate::pipeline::run_to_stdout(updates()?, State::new(symbols)?)
}
