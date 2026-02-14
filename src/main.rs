mod config;
mod detector;
mod icon;
mod notifier;

use clap::{Parser, Subcommand};
use config::{Config, NotificationMode};
use detector::{Detector, DetectorEvent};
use notifier::{CompositeNotifier, DialogNotifier, NotificationCenterNotifier, Notifier};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser)]
#[command(name = "yumetouch", about = "macOS YubiKey touch notifier")]
struct Cli {
    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Override notification mode
    #[arg(long)]
    mode: Option<NotificationMode>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install LaunchAgent for auto-start on login
    Install,
    /// Uninstall LaunchAgent
    Uninstall,
}

fn build_notifier(mode: &NotificationMode, sound: &str) -> Box<dyn Notifier> {
    let icon_path = icon::ensure_icon().to_str().map(String::from);

    match mode {
        NotificationMode::Notification => {
            Box::new(NotificationCenterNotifier::new(sound, icon_path))
        }
        NotificationMode::Dialog => Box::new(DialogNotifier::new(sound, icon_path)),
        NotificationMode::Both => Box::new(CompositeNotifier::new(vec![
            Box::new(NotificationCenterNotifier::new(sound, icon_path.clone())),
            Box::new(DialogNotifier::new(sound, icon_path)),
        ])),
    }
}

fn run_daemon(cli: &Cli) {
    let cfg = Config::load(cli.config.as_ref());
    let mode = cli.mode.as_ref().unwrap_or(&cfg.notification.mode);
    let sound = &cfg.notification.sound;

    log::info!("starting yumetouch (mode={mode}, sound={sound})");

    let mut notifier = build_notifier(mode, sound);
    let shutdown = Arc::new(AtomicBool::new(false));

    // Register signal handlers
    {
        let shutdown = shutdown.clone();
        ctrlc::set_handler(move || {
            log::info!("received shutdown signal");
            shutdown.store(true, Ordering::Relaxed);
        })
        .expect("failed to set signal handler");
    }

    let mut detector = Detector::new(shutdown);

    detector.run(|event| match event {
        DetectorEvent::TouchStarted => {
            log::info!("touch started — notifying");
            notifier.notify_touch_needed();
        }
        DetectorEvent::TouchCompleted => {
            log::info!("touch completed — dismissing");
            notifier.dismiss();
        }
    });

    log::info!("yumetouch shutting down");
}

fn install_launch_agent() {
    let binary = std::env::current_exe().expect("could not determine current executable path");
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
            <key>Label</key>
            <string>net.sadmelon.yumetouch</string>
            <key>ProgramArguments</key>
            <array>
                <string>{}</string>
            </array>
            <key>RunAtLoad</key>
            <true/>
            <key>KeepAlive</key>
            <true/>
            <key>StandardOutPath</key>
            <string>/tmp/yumetouch.log</string>
            <key>StandardErrorPath</key>
            <string>/tmp/yumetouch.err</string>
        </dict>
        </plist>"#,
        binary.display()
    );

    let plist_dir = home_dir().join("Library/LaunchAgents");
    std::fs::create_dir_all(&plist_dir).expect("could not create LaunchAgents directory");

    let plist_path = plist_dir.join("net.sadmelon.yumetouch.plist");
    std::fs::write(&plist_path, plist_content).expect("could not write plist file");

    // Unload first in case it's already loaded (ignore errors)
    let _ = std::process::Command::new("launchctl")
        .args(["unload", "-w"])
        .arg(&plist_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    let status = std::process::Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist_path)
        .status()
        .expect("failed to run launchctl");

    if status.success() {
        println!(
            "installed and loaded LaunchAgent at {}",
            plist_path.display()
        );
    } else {
        eprintln!("launchctl load failed (exit code: {:?})", status.code());
        std::process::exit(1);
    }
}

fn uninstall_launch_agent() {
    let plist_path = home_dir().join("Library/LaunchAgents/net.sadmelon.yumetouch.plist");

    if !plist_path.exists() {
        eprintln!("LaunchAgent not found at {}", plist_path.display());
        std::process::exit(1);
    }

    let status = std::process::Command::new("launchctl")
        .args(["unload", "-w"])
        .arg(&plist_path)
        .status()
        .expect("failed to run launchctl");

    if !status.success() {
        eprintln!("launchctl unload failed (exit code: {:?})", status.code());
    }

    std::fs::remove_file(&plist_path).expect("could not remove plist file");
    println!("uninstalled LaunchAgent");
}

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME not set"))
}

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(if cli.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .format_timestamp_secs()
        .init();

    match &cli.command {
        Some(Commands::Install) => install_launch_agent(),
        Some(Commands::Uninstall) => uninstall_launch_agent(),
        None => run_daemon(&cli),
    }
}
