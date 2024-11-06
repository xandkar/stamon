use std::time::Duration;

use anyhow::{anyhow, Result};
use x11::xlib;

const XKB_SYMBOLS_NAME_MASK: u32 = 1 << 2; // TODO Find in X11 lib.
const XKB_USE_CORE_KBD: u32 = 0x0100; // TODO Find in X11 lib.

/// Ref: <https://xwindow.angelfire.com/page2.html>
/// Ref: <https://www.oreilly.com/library/view/xlib-reference-manual/9780937175262/14_appendix-f.html>
pub struct X11 {
    display_ptr: *mut xlib::Display,
}

impl X11 {
    pub fn init() -> Result<Self> {
        let display_ptr = unsafe { xlib::XOpenDisplay(std::ptr::null()) };
        if display_ptr.is_null() {
            Err(anyhow!("XOpenDisplay failed"))
        } else {
            Ok(Self { display_ptr })
        }
    }

    fn symbols(&self) -> Result<String> {
        let desc_ptr = unsafe { xlib::XkbAllocKeyboard() };
        if desc_ptr.is_null() {
            return Err(anyhow!(
                "XkbAllocKeyboard: Failed to allocate keyboard"
            ));
        }
        if unsafe {
            xlib::XkbGetNames(
                self.display_ptr,
                XKB_SYMBOLS_NAME_MASK,
                desc_ptr,
            )
        } > 0
        {
            return Err(anyhow!(
                "XkbGetNames: Failed to retrieve key symbols"
            ));
        }
        let symbols_ptr = unsafe {
            xlib::XGetAtomName(self.display_ptr, (*(*desc_ptr).names).symbols)
        };
        if symbols_ptr.is_null() {
            return Err(anyhow!("XGetAtomName: Failed to get atom name"));
        }
        let symbols = unsafe { std::ffi::CStr::from_ptr(symbols_ptr) }
            .to_str()?
            .to_string();
        unsafe {
            xlib::XFree(symbols_ptr as *mut _);
            xlib::XkbFreeKeyboard(desc_ptr, XKB_SYMBOLS_NAME_MASK, 1);
        }
        Ok(symbols)
    }

    fn current_group_index(&self) -> Result<usize> {
        let mut state: std::mem::MaybeUninit<xlib::XkbStateRec> =
            std::mem::MaybeUninit::uninit();
        if unsafe {
            xlib::XkbGetState(
                self.display_ptr,
                XKB_USE_CORE_KBD,
                state.assume_init_mut(),
            )
        } > 0
        {
            return Err(anyhow!(
                "XkbGetState: Failed to retrieve keyboard state"
            ));
        }
        Ok(unsafe { state.assume_init() }.group as usize)
    }

    // TODO Avoid allocating new strings? Can we write into a passed buffer?
    pub fn keymap(&self) -> Result<String> {
        let symbols = self.symbols()?;
        let group_index = self.current_group_index()?;
        let symbol = symbols
            .split(['+', ':'])
            .filter(|s| !matches!(*s, "pc" | "evdev" | "inet" | "base"))
            .nth(group_index)
            .ok_or_else(|| anyhow!("group index not found in symbols"))?;
        let symbol = symbol.chars().take(2).collect::<String>();
        Ok(symbol)
    }
}

impl Drop for X11 {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(self.display_ptr);
        }
    }
}

struct State<'a> {
    prefix: &'a str,
    symbol: Option<String>,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            symbol: None,
        }
    }
}

impl<'a> crate::pipeline::State for State<'a> {
    type Event = String;

    fn update(
        &mut self,
        symbol: Self::Event,
    ) -> Result<Option<Vec<crate::alert::Alert>>> {
        self.symbol = Some(symbol);
        Ok(None)
    }

    fn display<W: std::io::Write>(&mut self, mut buf: W) -> Result<()> {
        let symbol = match self.symbol {
            None => "--",
            Some(ref s) => s,
        };
        writeln!(buf, "{}{}", self.prefix, symbol)?;
        Ok(())
    }
}

fn reads(interval: Duration, x11: &X11) -> impl Iterator<Item = String> + '_ {
    use crate::clock;

    clock::new(interval).filter_map(|clock::Tick| match x11.keymap() {
        Err(err) => {
            tracing::error!("Failure to lookup keymap: {:?}", err);
            None
        }
        Ok(symbol) => Some(symbol),
    })
}

pub fn run(prefix: &str, interval: Duration) -> Result<()> {
    let x11 = X11::init()?;
    crate::pipeline::run_to_stdout(reads(interval, &x11), State::new(prefix))
}
