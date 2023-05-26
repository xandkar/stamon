use std::path::{Path, PathBuf};

use anyhow::Result;

#[derive(Debug)]
struct Device {
    max: PathBuf,
    cur: PathBuf,
}

impl Device {
    pub fn new(name: &str) -> Self {
        let base = "/sys/class/backlight/";
        let max = [base, name, "max_brightness"].iter().collect();
        let cur = [base, name, "brightness"].iter().collect();
        Self { max, cur }
    }

    pub fn read_cur_brightness_pct(&self) -> Result<Option<u64>> {
        let max: f32 = std::fs::read_to_string(&self.max)?.trim().parse()?;
        let cur: f32 = std::fs::read_to_string(&self.cur)?.trim().parse()?;
        Ok(crate::math::percentage_round(cur, max))
    }
}

struct Watcher {
    dev: Device,
    _watcher: notify::RecommendedWatcher, // XXX To keep it from being dropped.
    receiver: std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
}

impl Watcher {
    fn new(device_name: &str) -> Result<Self> {
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

    fn iter(&self) -> impl Iterator<Item = Result<u64>> + '_ {
        use notify::event::{
            DataChange, Event, EventKind::Modify, ModifyKind,
        };

        // Dummy event to trigger initial reading:
        std::iter::once(Ok(Event::new(Modify(ModifyKind::Data(
            DataChange::Any,
        )))))
        .chain(self.receiver.iter())
        .filter_map(|event_result| match event_result {
            Ok(Event {
                kind: Modify(ModifyKind::Data(DataChange::Any)),
                ..
            }) => match self.dev.read_cur_brightness_pct() {
                Ok(None) => None,
                Ok(Some(pct)) => Some(Ok(pct)),
                Err(e) => Some(Err(
                    e.context("failed to read backlight percentage")
                )),
            },
            Ok(_) => None,
            Err(e) => {
                Some(Err(anyhow::Error::from(e).context("watch event error")))
            }
        })
    }
}

pub struct State<'a> {
    prefix: &'a str,
    percentage: Option<u64>,
}

impl<'a> State<'a> {
    pub fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            percentage: None,
        }
    }
}

impl<'a> crate::State for State<'a> {
    type Msg = u64;

    fn update(
        &mut self,
        percentage: Self::Msg,
    ) -> Result<Option<Vec<Box<dyn crate::Alert>>>> {
        self.percentage = Some(percentage);
        Ok(None)
    }

    fn display<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        write!(buf, "{}", self.prefix)?;
        match self.percentage {
            None => write!(buf, "----")?,
            Some(pct) => write!(buf, "{:3.0}%", pct)?,
        }
        writeln!(buf)?;
        Ok(())
    }
}

pub fn run(device: &str, prefix: &str) -> Result<()> {
    crate::pipeline_to_stdout(
        Watcher::new(device)?.iter(),
        Box::new(|x| x),
        State::new(prefix),
    )
}
