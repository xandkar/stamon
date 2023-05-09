use std::path::{Path, PathBuf};

use anyhow::Result;

#[derive(Debug)]
struct Device {
    max: PathBuf,
    cur: PathBuf,
}

impl Device {
    pub fn new(name: &str) -> Self {
        let dev: PathBuf = ["/sys/class/backlight/", name].iter().collect();
        let mut max = dev.clone();
        let mut cur = dev.clone();
        max.push("max_brightness");
        cur.push("brightness");
        Self { max, cur }
    }

    pub fn read_cur_brightness_pct(&self) -> Result<f32> {
        let max: f32 = std::fs::read_to_string(&self.max)?.trim().parse()?;
        let cur: f32 = std::fs::read_to_string(&self.cur)?.trim().parse()?;
        Ok(cur / max * 100.0)
    }
}

pub struct Watcher {
    dev: Device,
    _watcher: notify::RecommendedWatcher, // XXX To keep it from being dropped.
    receiver: std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
}

impl Watcher {
    pub fn new(device_name: &str) -> Result<Self> {
        let dev = Device::new(device_name);
        tracing::info!(
            "Instantiating new watcher for backlight device: {:?}",
            &dev
        );
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut _watcher = notify::recommended_watcher(sender)?;
        {
            use notify::Watcher;
            _watcher.watch(
                Path::new(&dev.cur),
                notify::RecursiveMode::Recursive,
            )?;
        }
        Ok(Self {
            dev,
            _watcher,
            receiver,
        })
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Result<f32>> + 'a {
        std::iter::once(self.dev.read_cur_brightness_pct()).chain(
            self.receiver.iter().filter_map(|event_result| {
                use notify::event::{
                    DataChange, Event, EventKind::Modify, ModifyKind,
                };
                match event_result {
                    Ok(Event {
                        kind: Modify(ModifyKind::Data(DataChange::Any)),
                        ..
                    }) => Some(self.dev.read_cur_brightness_pct()),
                    Ok(_) => None,
                    Err(e) => {
                        Some(Err(anyhow::Error::from(e)
                            .context("watch event error")))
                    }
                }
            }),
        )
    }
}
