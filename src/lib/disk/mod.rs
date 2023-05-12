use std::ffi::CString;
use std::mem::MaybeUninit;

use anyhow::{anyhow, Result};

pub fn usage(path: &str) -> Result<f32> {
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
    let used_percentage = (used as f32) / (total as f32) * 100.0;
    Ok(used_percentage)
}
