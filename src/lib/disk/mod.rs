use std::mem::MaybeUninit;
use std::{ffi::CString, time::Duration};

use anyhow::{anyhow, Result};

fn usage(path: &str) -> Result<Option<u64>> {
    let path = CString::new(path)?;
    let mut buf: MaybeUninit<libc::statfs> = MaybeUninit::uninit();
    let (total, free) = unsafe {
        match libc::statfs(path.as_ptr() as *const i8, buf.assume_init_mut())
        {
            0 => {
                let libc::statfs {
                    f_blocks: total,
                    f_bfree: free,
                    ..
                } = buf.assume_init();
                Ok((total, free))
            }
            n => Err(anyhow!("libc::statfs failed with {}", n)),
        }
    }?;
    let used = total - free;
    Ok(crate::math::percentage_ceiling(used as f32, total as f32))
}

struct State<'a> {
    prefix: &'a str,
    usage: Option<u64>,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            usage: None,
        }
    }
}

impl<'a> crate::State for State<'a> {
    type Event = Option<u64>;

    fn update(
        &mut self,
        msg: Self::Event,
    ) -> Result<Option<Vec<Box<dyn crate::Alert>>>> {
        self.usage = msg;
        Ok(None)
    }

    fn display<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        write!(buf, "{}", self.prefix)?;
        match self.usage {
            None => write!(buf, "----")?,
            Some(pct) => write!(buf, "{:3.0}%", pct)?,
        }
        writeln!(buf)?;
        Ok(())
    }
}

fn reads(
    interval: Duration,
    path: &str,
) -> impl Iterator<Item = Option<u64>> + '_ {
    use crate::clock;
    clock::new(interval).filter_map(|clock::Tick| match usage(path) {
        Err(err) => {
            tracing::error!("Failed to read disk usage: {:?}", err);
            None
        }
        Ok(usage_opt) => Some(usage_opt),
    })
}

pub fn run<'a>(
    prefix: &'a str,
    interval: Duration,
    path: &'a str,
) -> Result<()> {
    crate::pipeline_to_stdout(reads(interval, path), State::new(prefix))
}
