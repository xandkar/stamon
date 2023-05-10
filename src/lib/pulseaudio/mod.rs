use std::collections::HashSet;

use anyhow::{anyhow, Result};

pub enum Event {
    New(u64),
    Remove(u64),
}

fn name_to_num(name: &str) -> Result<u64> {
    if name.starts_with("#") {
        let id = name[1..].parse()?;
        Ok(id)
    } else {
        Err(anyhow!("Source name in unexpected format: {:?}", name))
    }
}

pub fn source_outputs_subscribe(
) -> Result<impl Iterator<Item = Result<Event>>> {
    let updates = crate::process::spawn("pactl", &["subscribe"])?.filter_map(
        |line_result| match line_result {
            Ok(line) => {
                match line.split_whitespace().collect::<Vec<&str>>()[..] {
                    ["Event", event, "on", "source-output", name] => {
                        match (event, name_to_num(name)) {
                            ("'new'", Ok(id)) => Some(Ok(Event::New(id))),
                            ("'remove'", Ok(id)) => {
                                Some(Ok(Event::Remove(id)))
                            }
                            (_, Err(e)) => Some(Err(e)),
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

pub fn source_outputs_list() -> Result<impl Iterator<Item = Result<Event>>> {
    let pactl_list =
        crate::process::exec("pactl", &["list", "source-outputs"])?;
    let mut sources = HashSet::new();
    for line in std::str::from_utf8(&pactl_list)?.lines() {
        tracing::info!("line: {:?}", &line);
        match line.split_whitespace().collect::<Vec<&str>>()[..] {
            ["Source", "Output", name] => {
                sources.insert(name_to_num(name)?);
            }
            _ => (),
        }
    }
    Ok(sources.into_iter().map(|id| Ok(Event::New(id))))
}
