// build.rs - abort build if required system commands are unavailable.

use std::process::{Command, Stdio};

/// List of (command, package) pairs required at build time.
/// This slice can be extended to add more command/package pairs.
const REQUIRED: &[(&str, &str)] = &[
    ("notify-send", "libnotify-bin"),
    ("xbacklight", "xbacklight"),
];

/// Returns true if `cmd` is available in PATH.
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Formats items into a comma-separated list with 'and' before the last item.
fn human_list(items: &[&str]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].to_string(),
        _ => {
            let mut s = items[..items.len() - 1].join(", ");
            s.push_str(" and ");
            s.push_str(items[items.len() - 1]);
            s
        }
    }
}

/// Build-script entry point.
///
/// Checks for required external commands and panics if any are missing.
fn main() {
    let missing = REQUIRED
        .iter()
        .filter(|&&(cmd, _)| !command_exists(cmd))
        .cloned()
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return;
    }

    let cmds = missing.iter().map(|&(cmd, _)| cmd).collect::<Vec<_>>();
    let pkgs = missing.iter().map(|&(_, pkg)| pkg).collect::<Vec<_>>();

    panic!(
        "Missing command(s): {}. Please install {} before proceeding to build.",
        human_list(&cmds),
        human_list(&pkgs)
    );
}
