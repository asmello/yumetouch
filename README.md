# yumetouch

A lightweight macOS daemon that detects when your YubiKey is waiting for a physical touch and shows a notification.

When a YubiKey with touch-required policy is used for GPG signing (e.g. `git commit -S`) or SSH auth (e.g. `git push` via gpg-agent), the YubiKey's LED blinks to indicate it needs touch. This is easy to miss, causing operations to time out. yumetouch watches for this and alerts you.

## How it works

yumetouch monitors the macOS unified log for messages from `usbsmartcardreaderd`. When the YubiKey is waiting for touch during any OpenPGP operation, macOS logs `"Time extension received"` repeatedly via CryptoTokenKit. This covers both GPG and SSH operations since they all flow through:

```
gpg/ssh → gpg-agent → scdaemon → PC/SC → usbsmartcardreaderd → YubiKey
```

When detected, yumetouch fires a notification. After 5 seconds of silence (touch was provided or the operation timed out), it dismisses.

## Installation

### Build from source

```bash
cargo build --release
cp target/release/yumetouch /usr/local/bin/
```

### Auto-start on login

```bash
yumetouch install
```

This installs a LaunchAgent that starts yumetouch at login and keeps it running. To remove it:

```bash
yumetouch uninstall
```

## Usage

Run in the foreground:

```bash
yumetouch
```

With verbose logging:

```bash
yumetouch -v
```

Override the notification mode:

```bash
yumetouch --mode dialog
```

Specify a config file:

```bash
yumetouch --config /path/to/config.toml
```

## Configuration

Create `~/.config/yumetouch/config.toml`:

```toml
[notification]
mode = "notification"  # "notification", "dialog", or "both"
sound = "Funk"         # macOS system sound name
timeout_secs = 5       # seconds of silence before dismissing
```

### Notification modes

| Mode | Description |
|------|-------------|
| `notification` | Notification Center banner with sound (default) |
| `dialog` | Modal dialog via osascript with sound, auto-dismisses after 30s |
| `both` | Both notification banner and modal dialog |

### Available sounds

Any sound in `/System/Library/Sounds/` works. Common choices: `Funk`, `Glass`, `Ping`, `Pop`, `Tink`, `Blow`, `Bottle`, `Frog`, `Hero`, `Morse`, `Purr`, `Sosumi`, `Submarine`.

## Troubleshooting

### No notifications appear

`log stream` may require granting your terminal (or the LaunchAgent) **Full Disk Access** in System Preferences > Privacy & Security > Full Disk Access.

### Testing detection

Trigger a GPG operation that requires touch:

```bash
echo test | gpg --sign > /dev/null
```

Run yumetouch with `-v` to see debug output confirming it detects the log messages.

### Checking LaunchAgent logs

```bash
tail -f /tmp/yumetouch.log
tail -f /tmp/yumetouch.err
```

## License

MIT
