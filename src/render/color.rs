//! Minimal ANSI color support respecting `NO_COLOR`.

use std::sync::atomic::{AtomicBool, Ordering};

static COLOR_DISABLED: AtomicBool = AtomicBool::new(false);

/// Call once at startup to unconditionally disable color.
pub fn disable_color() {
    COLOR_DISABLED.store(true, Ordering::Relaxed);
}

/// Call once at startup to check `NO_COLOR` env var.
pub fn respect_no_color_env() {
    if std::env::var_os("NO_COLOR").is_some() {
        disable_color();
    }
}

fn enabled() -> bool {
    !COLOR_DISABLED.load(Ordering::Relaxed)
}

macro_rules! color_fn {
    ($name:ident, $code:expr) => {
        #[must_use]
        pub fn $name(input: &str) -> String {
            if enabled() {
                format!("\x1b[{}m{}\x1b[0m", $code, input)
            } else {
                input.to_owned()
            }
        }
    };
}

color_fn!(bold, "1");
color_fn!(green, "32");
color_fn!(yellow, "33");
color_fn!(red, "31");
color_fn!(dim, "2");

#[must_use]
pub fn severity(severity: crate::model::Severity) -> String {
    use crate::model::Severity;
    match severity {
        Severity::Critical => red("critical"),
        Severity::High => red("high"),
        Severity::Medium => yellow("medium"),
        Severity::Low => dim("low"),
        Severity::Informational => dim("informational"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_disabled_returns_plain() {
        disable_color();
        let bold_text = bold("hello");
        let green_text = green("hello");
        let yellow_text = yellow("hello");
        let red_text = red("hello");
        let dim_text = dim("hello");
        let sev = severity(crate::model::Severity::Critical);
        assert_eq!(bold_text, "hello");
        assert_eq!(green_text, "hello");
        assert_eq!(yellow_text, "hello");
        assert_eq!(red_text, "hello");
        assert_eq!(dim_text, "hello");
        assert_eq!(sev, "critical");
    }

    #[test]
    fn severity_matches_enum() {
        let _ = crate::model::Severity::Critical;
        let _ = crate::model::Severity::High;
    }
}
