# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
cargo build              # Build debug binary
cargo build --release    # Build release binary
cargo clippy -- -D warnings  # Lint (CI enforces zero warnings)
cargo fmt --check        # Check formatting
cargo fmt                # Auto-format
cargo install --path .   # Install locally
```

Nix is also supported (`nix build`, `nix develop`, `nix fmt -- --ci`).

## Architecture

macOS-only Rust daemon that detects YubiKey FIDO2 touch requests via process polling and shows notifications.

**Main loop** (`main.rs`): Parses CLI args (clap), loads config, wires up the detector → notifier pipeline. Also handles `install`/`uninstall` subcommands for LaunchAgent management.

**Detection** (`detector.rs`): Polls every 300ms via `pgrep -f` for `ssh-keygen.*-Y sign` (git signing) or `ssh-sk-helper` (SSH auth). Uses a 3-state machine: `Idle` → `MaybePending` → `TouchPending`, with a 500ms grace period to avoid false positives from non-FIDO2 keys.

**Notification** (`notifier.rs`): `Notifier` trait with three implementations:
- `NotificationCenterNotifier` — uses `mac-notification-sys` crate for banner notifications
- `DialogNotifier` — spawns `osascript -l JavaScript` (JXA) to show a native `NSPanel` with custom icon. Manages child processes and kills them on dismiss.
- `CompositeNotifier` — wraps multiple notifiers for "both" mode

**Icon** (`icon.rs`): Embeds `resources/icon.png` at compile time via `include_bytes!`, extracts to `~/Library/Caches/yumetouch/icon.png` at runtime (required by notification APIs that take file paths).

**Config** (`config.rs`): TOML config from `~/.config/yumetouch/config.toml` with serde. Falls back to defaults if missing/invalid.

## Key Constraints

- macOS only — depends on `osascript`, `pgrep`, `afplay`, `launchctl`, and `mac-notification-sys`
- Rust edition 2024
- No test suite currently exists
- The JXA dialog script in `notifier.rs` is inline JavaScript — `osascript` requires `setActivationPolicy(1)` (accessory) to show windows
