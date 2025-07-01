use clap::Parser;
use std::process::Command;

/// Command-line arguments for the brightness notifier.
///
/// Provides options to increase, decrease, set, or get the current
/// brightness level, along with notification timing and fade settings.
#[derive(Parser)]
#[command(
    author = "Manuel Albisu-Bouza",
    version = "1.1.0",
    about = "Brightness Notifier for LXQt",
    long_about = "A simple CLI tool that controls screen brightness \
                  and displays a desktop notification for LXQt using \
                  libnotify and xbacklight."
)]
struct Args {
    /// Increase brightness by PERCENTAGE
    ///
    /// [default: 5]
    #[arg(
        short = 'i',
        long = "increase",
        value_name = "PERCENTAGE",
        num_args = 0..=1,
        default_missing_value = "5",
        conflicts_with_all = &["set", "get"]
    )]
    increase: Option<u8>,

    /// Decrease brightness by PERCENTAGE
    ///
    /// [default: 5]
    #[arg(
        short = 'd',
        long = "decrease",
        value_name = "PERCENTAGE",
        num_args = 0..=1,
        default_missing_value = "5",
        conflicts_with_all = &["set", "get"]
    )]
    decrease: Option<u8>,

    /// Set brightness to PERCENTAGE
    ///
    /// [range: 1-100]
    #[arg(
        short = 's',
        long = "set",
        value_name = "PERCENTAGE",
        value_parser = clap::value_parser!(u8).range(1..=100),
        conflicts_with_all = &["increase", "decrease", "get"]
    )]
    set: Option<u8>,

    /// Show current brightness without changes
    #[arg(
        short = 'g',
        long = "get",
        conflicts_with_all = &["increase", "decrease", "set"]
    )]
    get: bool,

    /// Notification timeout in milliseconds
    ///
    /// [range: 0-120000]
    #[arg(
        short = 't',
        long = "timeout",
        value_name = "DURATION",
        default_value_t = 2000,
        value_parser = clap::value_parser!(u32).range(0..=120000)
    )]
    timeout: u32,

    /// Fade time in milliseconds
    ///
    /// [range: 0-60000]
    #[arg(
        short = 'f',
        long = "fade",
        value_name = "TIME",
        default_value_t = 100,
        value_parser = clap::value_parser!(u16).range(0..=60000)
    )]
    fade_time: u16,

    /// Number of steps in fade
    ///
    /// [range: 1-200]
    #[arg(
        short = 'p',
        long = "steps",
        value_name = "STEPS",
        default_value_t = 25,
        value_parser = clap::value_parser!(u8).range(1..=200)
    )]
    steps: u8,
}

/// Exit the program with a success or failure code.
///
/// # Arguments
///
/// * `success` - If true, exit with code 0; otherwise exit with code 1.
fn exit_with(success: bool) -> ! {
    std::process::exit(if success { 0 } else { 1 });
}

/// Run an xbacklight command with the specified mode and value.
///
/// # Arguments
///
/// * `mode`  - The xbacklight mode flag (e.g., "-set", "-inc", "-dec").
/// * `value` - The brightness percentage value.
/// * `args`  - Command-line arguments containing timing parameters.
///
/// # Returns
///
/// `true` if the command is successful; `false` otherwise.
fn run_brightness_cmd(mode: &str, value: u8, args: &Args) -> bool {
    let v = value.to_string();
    let t = args.fade_time.to_string();
    let s = args.steps.to_string();

    Command::new("xbacklight")
        .args(&[
            mode, &v,
            "-time", &t,
            "-steps", &s
        ])
        .status()
        .map_or(false, |st| st.success())
}

/// Get the current screen brightness as a percentage.
///
/// Uses xbacklight to query the current brightness level.
///
/// # Returns
///
/// `Some(u8)` containing the brightness rounded to the nearest percent,
/// or `None` if the command failed or output could not be parsed.
fn get_current_brightness() -> Option<u8> {
    let out = Command::new("xbacklight")
        .arg("-get")
        .output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = std::str::from_utf8(&out.stdout).ok()?;
    let v = s.trim().parse::<f32>().ok()?;
    Some(v.round() as u8)
}

/// Choose the appropriate icon based on the brightness percentage.
///
/// # Arguments
///
/// * `brightness` - Brightness percentage (0-100).
///
/// # Returns
///
/// A string slice corresponding to the appropriate symbolic icon name.
fn icon_for(brightness: u8) -> &'static str {
    match brightness {
        0..=32  => "display-brightness-low-symbolic",
        33..=65 => "display-brightness-medium-symbolic",
        _       => "display-brightness-high-symbolic",
    }
}

/// Display the current brightness in a desktop notification.
///
/// Uses notify-send to show a summary with the brightness percentage.
///
/// # Arguments
///
/// * `timeout` - Notification timeout in milliseconds.
///
/// # Returns
///
/// `Some(u8)` containing the brightness if successful, or `None` on failure.
fn display_notification(timeout: u32) -> Option<u8> {
    let brightness = get_current_brightness()?;
    let summary    = format!("Brightness: {}%", brightness);
    let icon       = icon_for(brightness);
    let t          = timeout.to_string();

    Command::new("notify-send")
        .args(&[
            "--app-name",   "lxqt-brightness",
            "--icon",       icon,
            "--replace-id", "1",
            "--expire-time", &t,
            &summary,
        ])
        .status()
        .ok()?
        .success()
        .then(|| {
            println!("Current brightness: {}%", brightness);
            brightness
        })
}

/// Adjust the display's brightness level based on the provided arguments.
///
/// Increases, decreases, or sets the brightness, ensuring it never drops
/// below 1%.
///
/// # Arguments
///
/// * `args` - Parsed command-line arguments.
///
/// # Returns
///
/// `true` if the operation is successful; `false` otherwise.
fn adjust_brightness(args: &Args) -> bool {
    let current = get_current_brightness();

    let (mode, value) = if let Some(inc) = args.increase {
        if current.map_or(false, |c| c <= 1) {
            ("-set", inc)
        } else {
            ("-inc", inc)
        }
    } else if let Some(dec) = args.decrease {
        let val = current
            .map(|c| c.saturating_sub(dec).max(1))
            .unwrap_or(dec);
        ("-set", val)
    } else {
        return true;
    };

    run_brightness_cmd(mode, value, args)
}

/// Set the display's brightness level to a specified value.
///
/// # Arguments
///
/// * `brightness` - Desired brightness percentage (1-100).
/// * `args`       - Parsed command-line arguments.
///
/// # Returns
///
/// `true` if successful; `false` otherwise.
fn set_brightness(brightness: u8, args: &Args) -> bool {
    run_brightness_cmd("-set", brightness, args)
}

/// Program entry point.
///
/// Parses arguments and executes the requested action, then displays
/// a notification on exit.
fn main() {
    let args = Args::parse();

    if args.get {
        exit_with(display_notification(args.timeout).is_some());
    }

    if let Some(value) = args.set {
        if !set_brightness(value, &args) {
            eprintln!("Failed to set brightness to {}.", value);
            exit_with(false);
        }
    } else if (args.increase.is_some() || args.decrease.is_some())
        && !adjust_brightness(&args)
    {
        eprintln!("Failed to adjust the brightness.");
        exit_with(false);
    }

    exit_with(display_notification(args.timeout).is_some());
}
