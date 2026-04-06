// Respects NO_COLOR (https://no-color.org/) standard.
// Set NO_COLOR=1 to disable all color output.

/// Check if color output is enabled.
pub fn color_enabled() -> bool {
    std::env::var("NO_COLOR").is_err()
}

pub const RESET: &str = "\x1b[0m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";

pub const GROUP_COLORS: &[&str] = &[
    "\x1b[48;5;24m\x1b[97m",  // dark blue
    "\x1b[48;5;22m\x1b[97m",  // dark green
    "\x1b[48;5;94m\x1b[97m",  // dark orange
    "\x1b[48;5;30m\x1b[97m",  // teal
    "\x1b[48;5;238m\x1b[97m", // dark gray
];

/// Get a color code, returning empty string if colors are disabled.
pub fn color(code: &str) -> &str {
    if color_enabled() { code } else { "" }
}
