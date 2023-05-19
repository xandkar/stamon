// TODO Rewrite with pulseaudio bindings.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use anyhow::{anyhow, Result};

pub type Seq = u64;

pub struct Sink<'a> {
    _seq: Seq,
    pub name: &'a str,
    pub mute: bool,
    pub vol_left: u64,
    pub vol_right: u64,
}

pub enum Event {
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

pub enum Stream {
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

pub type Update = (Event, Stream, Seq);

pub struct Updates {
    // TODO init_sinks: Vec<Seq>, // Maybe by name rather than seq?
    init_source_outputs: Vec<Seq>,
}

impl Updates {
    pub fn new() -> Result<Self> {
        Ok(Self {
            init_source_outputs: source_outputs_list()?,
        })
    }

    pub fn iter(&self) -> Result<impl Iterator<Item = Result<Update>> + '_> {
        let init_vol_change =
            std::iter::once(Ok((Event::Change, Stream::Sink, 0)));
        let init_source_outputs = self
            .init_source_outputs
            .iter()
            .map(|seq| Ok((Event::New, Stream::SourceOutput, *seq)));
        let updates = init_vol_change
            .chain(init_source_outputs)
            .chain(subscribe()?);
        Ok(updates)
    }
}

pub enum Volume {
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

pub struct State {
    source_outputs: HashSet<Seq>,
    volume: Volume, // TODO Maybe Option instead of init fetch?
}

impl State {
    pub fn new() -> Result<Self> {
        let source_outputs = HashSet::new();
        let volume = Volume::fetch()?;
        Ok(Self {
            source_outputs,
            volume,
        })
    }

    pub fn update(&mut self, update: Update) -> Result<()> {
        match update {
            (Event::Change, Stream::Sink, _) => {
                self.volume = Volume::fetch()?;
            }
            (Event::New, Stream::SourceOutput, seq) => {
                self.source_outputs.insert(seq);
            }
            (Event::Remove, Stream::SourceOutput, seq) => {
                self.source_outputs.remove(&seq);
            }
            _ => (),
        }
        Ok(())
    }

    pub fn write<W: std::io::Write>(
        &self,
        mut buf: W,
        prefix: &str,
        symbol_mic_on: &str,
        symbol_mic_off: &str,
    ) -> Result<()> {
        let sym_mute = "  X  ";
        let sym_mic = if self.source_outputs.is_empty() {
            symbol_mic_off
        } else {
            symbol_mic_on
        };
        let sym_eq = "=";
        let sym_ap = "~";
        match self.volume {
            Volume::Muted => {
                writeln!(buf, "{}{} {}", prefix, sym_mute, sym_mic)?
            }
            Volume::Exactly(n) => {
                writeln!(buf, "{}{}{:3}% {}", prefix, sym_eq, n, sym_mic)?
            }
            Volume::Approx(n) => {
                writeln!(buf, "{}{}{:3}% {}", prefix, sym_ap, n, sym_mic)?
            }
        }
        Ok(())
    }
}

pub fn subscribe() -> Result<impl Iterator<Item = Result<Update>>> {
    let updates = crate::process::spawn("pactl", &["subscribe"])?.filter_map(
        |line_result| match line_result {
            Ok(line) => {
                // TODO Log invalidly-formatted event lines?
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
                    _ => None,
                }
            }
            Err(e) => Some(Err(anyhow::Error::from(e))),
        },
    );
    Ok(updates)
}

fn seq_parse(name: &str) -> Option<Seq> {
    name.strip_prefix('#').and_then(|seq| seq.parse().ok())
}

pub fn pactl_list_sinks_parse<'a>(data: &'a str) -> Result<Vec<Sink<'a>>> {
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
            ["Volume:", "front-left:", _, "/", left, "/", _, "dB,", "front-right:", _, "/", right, "/", _, "dB"]
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
    s.strip_suffix('%').and_then(|s| s.parse().ok())
}

fn source_outputs_list() -> Result<Vec<Seq>> {
    let pactl_list =
        crate::process::exec("pactl", &["list", "source-outputs"])?;
    let mut sources: HashSet<Seq> = HashSet::new();
    for line in std::str::from_utf8(&pactl_list)?.lines() {
        if let ["Source", "Output", seq] =
            line.split_whitespace().collect::<Vec<&str>>()[..]
        {
            if let Some(seq) = seq_parse(seq) {
                sources.insert(seq);
            }
        }
    }
    Ok(sources.into_iter().collect())
}
