mod x11 {
    use anyhow::{anyhow, Result};
    use x11::xlib;

    const XKB_SYMBOLS_NAME_MASK: u32 = 1 << 2; // TODO Find in X11 lib.
    const XKB_USE_CORE_KBD: u32 = 0x0100; // TODO Find in X11 lib.

    pub struct X11 {
        display_ptr: *mut xlib::_XDisplay,
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
            unsafe {
                let desc_ptr = xlib::XkbAllocKeyboard();
                if desc_ptr.is_null() {
                    return Err(anyhow!(
                        "XkbAllocKeyboard: Failed to allocate keyboard"
                    ));
                }
                if xlib::XkbGetNames(
                    self.display_ptr,
                    XKB_SYMBOLS_NAME_MASK,
                    desc_ptr,
                ) > 0
                {
                    return Err(anyhow!(
                        "XkbGetNames: Failed to retrieve key symbols"
                    ));
                }
                let symbols_ptr = xlib::XGetAtomName(
                    self.display_ptr,
                    (*(*desc_ptr).names).symbols,
                );
                if symbols_ptr.is_null() {
                    return Err(anyhow!(
                        "XGetAtomName: Failed to get atom name"
                    ));
                }
                let symbols = std::ffi::CStr::from_ptr(symbols_ptr)
                    .to_str()?
                    .to_string();
                xlib::XFree(symbols_ptr as *mut _);
                xlib::XkbFreeKeyboard(desc_ptr, XKB_SYMBOLS_NAME_MASK, 1);
                Ok(symbols)
            }
        }

        fn current_group_index(&self) -> Result<usize> {
            unsafe {
                let mut state: std::mem::MaybeUninit<xlib::XkbStateRec> =
                    std::mem::MaybeUninit::uninit();
                if xlib::XkbGetState(
                    self.display_ptr,
                    XKB_USE_CORE_KBD,
                    state.assume_init_mut(),
                ) > 0
                {
                    return Err(anyhow!(
                        "XkbGetState: Failed to retrieve keyboard state"
                    ));
                }
                Ok(state.assume_init().group as usize)
            }
        }

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
}

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "1.0")]
    interval: f32,

    #[clap(long = "prefix", short = 'p', default_value = "")]
    prefix: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();
    let cli = Cli::parse();
    log::info!("cli: {:?}", &cli);
    let x11 = x11::X11::init()?;
    loop {
        match x11.keymap() {
            Ok(symbol) => println!("{}{}", &cli.prefix, symbol),
            Err(err) => log::error!("Failure to lookup keymap: {:?}", err),
        }
        std::thread::sleep(std::time::Duration::from_secs_f32(cli.interval));
    }
}
