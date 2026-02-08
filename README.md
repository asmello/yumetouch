# yumetouch

A lightweight macOS daemon that detects when your YubiKey is waiting for a physical touch and shows a notification.

When using a FIDO2 YubiKey (e.g. `ed25519-sk`) for SSH authentication or Git commit signing, the key blinks to indicate it needs touch. This is easy to miss, causing operations to hang. yumetouch watches for this and alerts you.

## How it works

yumetouch polls for processes that indicate a YubiKey touch is pending:

- `ssh-keygen -Y sign` — Git commit/tag signing via SSH key
- `ssh-sk-helper` — SSH authentication with FIDO2 keys (git push, ssh connections)

When a signing or auth process persists beyond a short grace period (500ms), yumetouch assumes the YubiKey is waiting for touch and fires a notification. Once the process exits, it dismisses.

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
| `notification` | Notification Center banner with YubiKey icon and sound (default) |
| `dialog` | Modal alert with YubiKey icon and sound, dismisses on touch or OK |
| `both` | Both notification banner and modal alert |

### Available sounds

Any sound in `/System/Library/Sounds/` works. Common choices: `Funk`, `Glass`, `Ping`, `Pop`, `Tink`, `Blow`, `Bottle`, `Frog`, `Hero`, `Morse`, `Purr`, `Sosumi`, `Submarine`.

## Troubleshooting

### No notifications appear

Make sure you're using Homebrew's OpenSSH (`/opt/homebrew/bin/ssh`), not the macOS built-in. The built-in SSH doesn't use `ssh-sk-helper` in the same way.

### Testing detection

Trigger an SSH signing operation that requires touch:

```bash
echo test | ssh-keygen -Y sign -f ~/.ssh/id_ed25519_sk -n test
```

Run yumetouch with `-v` to see debug output confirming it detects the process.

### Checking LaunchAgent logs

```bash
tail -f /tmp/yumetouch.log
tail -f /tmp/yumetouch.err
```

## License

MIT
