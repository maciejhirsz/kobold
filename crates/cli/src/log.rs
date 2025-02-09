use std::fmt;
use std::sync::OnceLock;

static VERBOSE_OUTPUT: OnceLock<()> = OnceLock::new();

pub fn enable_verbose_output() {
    let _ = VERBOSE_OUTPUT.set(());
}

pub fn is_verbose_output_enabled() -> bool {
    VERBOSE_OUTPUT.get().is_some()
}

static COLOR_OUTPUT: OnceLock<()> = OnceLock::new();

pub fn enable_color_output() {
    let _ = COLOR_OUTPUT.set(());
}

pub fn is_color_output_enabled() -> bool {
    COLOR_OUTPUT.get().is_some()
}

pub mod color {
    use crossterm::style::{StyledContent, Stylize};

    pub type Color = fn(&str) -> StyledContent<&str>;

    pub const DARK_RED: Color = |s| s.dark_red().bold();
    pub const DARK_YELLOW: Color = |s| s.dark_yellow().bold();
    pub const DARK_BLUE: Color = |s| s.dark_blue().bold();
}

pub struct Print(pub &'static str, pub color::Color);

impl fmt::Display for Print {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Print(label, color) = self;
        if is_color_output_enabled() {
            write!(f, "{}", color(label))
        } else {
            write!(f, "{label}")
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        eprintln!("{}: {}", $crate::log::Print("error", $crate::log::color::DARK_RED), format_args!($($arg)*));
    }};
}

pub use error;

#[macro_export]
macro_rules! note {
    ($($arg:tt)*) => {{
        eprintln!("{}: {}", $crate::log::Print("note", $crate::log::color::DARK_BLUE), format_args!($($arg)*));
    }};
}

pub use note;

#[macro_export]
macro_rules! building {
    ($($arg:tt)*) => {{
        eprintln!("{} {}", $crate::log::Print("    Building", $crate::log::color::DARK_BLUE), format_args!($($arg)*));
    }};
}

pub use building;

#[macro_export]
macro_rules! optimized {
    ($($arg:tt)*) => {{
        eprintln!("{} {}", $crate::log::Print("   Optimized", $crate::log::color::DARK_BLUE), format_args!($($arg)*));
    }};
}

pub use optimized;

#[macro_export]
macro_rules! creating {
    ($($arg:tt)*) => {{
        eprintln!("{} {}", $crate::log::Print("    Creating", $crate::log::color::DARK_BLUE), format_args!($($arg)*));
    }};
}

pub use creating;

#[macro_export]
macro_rules! starting {
    ($($arg:tt)*) => {{
        eprintln!("{} {}", $crate::log::Print("    Starting", $crate::log::color::DARK_BLUE), format_args!($($arg)*));
    }};
}

pub use starting;

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        eprintln!("{} {}", $crate::log::Print("     Warning", $crate::log::color::DARK_YELLOW), format_args!($($arg)*));
    }};
}

pub use warning;

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        if $crate::log::is_verbose_output_enabled() {
            eprintln!("{} {}", $crate::log::Print("        Info", $crate::log::color::DARK_BLUE), format_args!($($arg)*));
        }
    }};
}

pub use info;
