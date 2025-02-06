use std::{
    ffi::{c_char, CString},
    mem::MaybeUninit,
    time::Duration,
};

use anyhow::{anyhow, Result};

fn usage(path: &str) -> Result<Option<u64>> {
    let path: CString = CString::new(path)?;
    let path: *const c_char = path.as_ptr();
    let mut buf: MaybeUninit<libc::statfs> = MaybeUninit::uninit();
    match unsafe { libc::statfs(path, buf.assume_init_mut()) } {
        0 => {
            let (total, free) = unsafe {
                let libc::statfs {
                    f_blocks: total,
                    f_bfree: free,
                    ..
                } = buf.assume_init();
                (total, free)
            };
            let used = total - free;
            let used_pct =
                crate::math::percentage_ceiling(used as f32, total as f32);
            Ok(used_pct)
        }
        n => Err(anyhow!("libc::statfs failed with {}", n)),
    }
}

struct State<'a> {
    prefix: &'a str,
    postfix: &'a str,
    usage: Option<u64>,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str, postfix: &'a str) -> Self {
        Self {
            prefix,
            postfix,
            usage: None,
        }
    }
}

impl<'a> crate::pipeline::State for State<'a> {
    type Event = Option<u64>;

    fn update(
        &mut self,
        msg: Self::Event,
    ) -> Result<Option<Vec<crate::alert::Alert>>> {
        self.usage = msg;
        Ok(None)
    }

    fn display<W: std::io::Write>(&mut self, mut buf: W) -> Result<()> {
        write!(buf, "{}", self.prefix)?;
        match self.usage {
            None => write!(buf, "----")?,
            Some(pct) => write!(buf, "{:3.0}%", pct)?,
        }
        writeln!(buf, "{}", self.postfix)?;
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
    postfix: &'a str,
    interval: Duration,
    path: &'a str,
) -> Result<()> {
    crate::pipeline::run_to_stdout(
        reads(interval, path),
        State::new(prefix, postfix),
    )
}
