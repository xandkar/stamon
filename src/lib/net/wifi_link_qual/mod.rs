use anyhow::{anyhow, Error, Result};

use std::{io::BufRead, time::Duration}; // .lines()

const PROC_NET_WIRELESS: &str = "/proc/net/wireless";

struct State<'a> {
    prefix: &'a str,
    link_qual: Option<u64>,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            link_qual: None,
        }
    }
}

impl<'a> crate::State for State<'a> {
    type Msg = Option<u64>;

    fn update(
        &mut self,
        link_qual: Self::Msg,
    ) -> Result<Option<Vec<Box<dyn crate::Alert>>>> {
        self.link_qual = link_qual;
        let alerts = None;
        Ok(alerts)
    }

    fn display<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        // TODO Tests
        write!(buf, "{}", self.prefix)?;
        match self.link_qual {
            Some(percentage) => {
                write!(buf, "{:3}%", percentage)?;
            }
            None => {
                // TODO User-configurable symbol?
                write!(buf, "----")?;
            }
        }
        writeln!(buf)?;
        Ok(())
    }
}

fn reader<'a, E>(
    interface: &'a str,
) -> Box<dyn 'a + Fn(E) -> Result<Option<u64>>> {
    Box::new(|_| match read(interface) {
        Ok(pct_opt) => Ok(pct_opt),
        Err(err) => {
            tracing::error!("Failed to read link quality: {:?}", err);
            Ok(None)
        }
    })
}

fn read(interface: &str) -> Result<Option<u64>> {
    let file = std::fs::File::open(PROC_NET_WIRELESS)?;
    let reader = std::io::BufReader::new(file);
    parse(reader.lines(), interface).map_err(Error::from)
}

fn parse(
    data_lines: impl Iterator<Item = Result<String, std::io::Error>>,
    interface: &str,
) -> Result<Option<u64>> {
    let mut line_num = 0;
    for line_result in data_lines {
        let line = line_result?;
        line_num += 1;
        if line_num > 2 && line.starts_with(interface) {
            let mut fields = line.split_whitespace();
            let cur = fields
                .nth(2)
                .ok_or_else(|| {
                    anyhow!("Missing link quality in line: {line:?}")
                })
                .and_then(|link_quality| {
                    link_quality.parse::<f32>().map_err(|_| {
                        anyhow!(
                            "Link quality value invalid: {:?}",
                            link_quality
                        )
                    })
                })?;
            // "The cfg80211 wext compat layer assumes a maximum quality of 70"
            // https://git.openwrt.org/?p=project/iwinfo.git;a=blob;f=iwinfo_nl80211.c
            let max = 70.0;
            return Ok(crate::math::percentage_floor(cur, max));
        }
    }
    Ok(None)
}

pub fn run(interval: Duration, interface: &str, prefix: &str) -> Result<()> {
    crate::pipeline_to_stdout(
        crate::clock::new(interval),
        reader(interface),
        State::new(prefix),
    )
}
