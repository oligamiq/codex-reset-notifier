# codex-reset-notifier

[![CI](https://github.com/oligamiq/codex-reset-notifier/actions/workflows/ci.yml/badge.svg)](https://github.com/oligamiq/codex-reset-notifier/actions/workflows/ci.yml)

A small Rust daemon that sends phone push notifications when a Codex quota window is **exhausted** or **reset**.

> [!IMPORTANT]
> This is an unofficial community project and is not affiliated with or endorsed by OpenAI.

It does **not** read or persist OpenAI credentials itself. It starts the locally installed `codex app-server`, uses Codex's existing login, and queries `account/rateLimits/read`.

Notification path:

```text
Codex app-server -> codex-reset-notifier -> ntfy -> Android/iOS
```

## Features

- Notifies once when an observed quota window reaches exhaustion.
- Notifies once when that quota becomes available again.
- Handles whatever primary/secondary quota windows Codex currently returns instead of assuming fixed windows.
- Polls at a low normal frequency (5 minutes recommended).
- Wakes just after a known reset deadline and briefly retries every 5 seconds if the server has not reflected the reset yet.
- Persists state so restarts do not lose the previous quota/deadline observation.
- Supports public ntfy, authenticated ntfy, and self-hosted ntfy.
- Uses the existing Codex login; no OpenAI API key is required by this program.

## Requirements

- A working `codex` CLI installation with an authenticated ChatGPT/Codex session.
- Rust toolchain for building from source.
- The ntfy Android/iOS app, or another client subscribed to your chosen ntfy topic.

The current implementation has been exercised against `codex-cli 0.146.0`. The app-server protocol can change between Codex releases, so please report compatibility regressions with the Codex CLI version included.

## Install

Linux x64 / macOS arm64:

```bash
curl -fsSL https://github.com/oligamiq/codex-reset-notifier/releases/latest/download/install.sh | sh
```

Windows x64 (PowerShell):

```powershell
irm https://github.com/oligamiq/codex-reset-notifier/releases/latest/download/install.ps1 | iex
```

The installers download the latest GitHub Release and verify its SHA256 against the published `SHA256SUMS.txt` before installing. Override the destination with `CODEX_NOTIFY_INSTALL_DIR`.

If you prefer to inspect the installer before executing it, download `install.sh` / `install.ps1` first and run it locally. Building from source is also supported:

```bash
cargo install --git https://github.com/oligamiq/codex-reset-notifier --locked
```

## Build

```bash
git clone https://github.com/oligamiq/codex-reset-notifier.git
cd codex-reset-notifier
cargo build --release
```

Check currently visible quota windows without sending a notification:

```bash
cargo run -- --dry-run --once
```

## Configure ntfy

Subscribe on your phone to a long, random topic, then set:

```bash
export CODEX_NOTIFY_NTFY_TOPIC='your-long-random-topic'
export CODEX_NOTIFY_INTERVAL_SECS=300
```

Optional settings:

```bash
# Self-hosted ntfy:
export CODEX_NOTIFY_NTFY_SERVER='https://ntfy.example.com'

# Authenticated ntfy:
export CODEX_NOTIFY_NTFY_TOKEN='tk_...'
```

Test push delivery:

```bash
cargo run -- --test-notification
```

Run continuously:

```bash
cargo run --release --
```

## Linux: systemd user service

Install the binary and service template:

```bash
install -Dm755 target/release/codex-reset-notifier ~/.local/bin/codex-reset-notifier
mkdir -p ~/.config/codex-reset-notifier ~/.config/systemd/user
cp .env.example ~/.config/codex-reset-notifier/env
cp packaging/systemd/codex-reset-notifier.service ~/.config/systemd/user/
```

Edit `~/.config/codex-reset-notifier/env`, then enable it:

```bash
systemctl --user daemon-reload
systemctl --user enable --now codex-reset-notifier.service
journalctl --user -u codex-reset-notifier.service -f
```

If you want the user service to start at boot even before interactive login, enable systemd user lingering for that account using your distribution's normal administration procedure.

## Detection behavior

### Exhaustion

A notification is emitted only when an already-observed window crosses from below the exhaustion threshold to exhausted. Remaining at exhausted does not repeatedly notify.

### Reset

A reset is emitted when either:

- an exhausted window becomes available again, or
- its reset timestamp advances after the previous deadline becomes due.

When the known reset is closer than the normal polling interval, the daemon sleeps until roughly two seconds after that deadline instead of waiting for the next five-minute poll. If the app-server still exposes the old deadline, it retries every five seconds for up to two minutes before returning to normal polling.

The state is updated after successful notification delivery. If ntfy delivery fails, the transition remains eligible for retry instead of being silently marked as delivered.

## State

State is stored at:

```text
$XDG_STATE_HOME/codex-reset-notifier/state.json
```

or, if `XDG_STATE_HOME` is unset:

```text
~/.local/state/codex-reset-notifier/state.json
```

The state contains quota percentages and reset timestamps, not Codex/OpenAI credentials.

## Security notes

- Treat a public ntfy topic name as a secret capability URL. Use a long random value.
- Do not commit your real topic or ntfy token.
- For stronger access control, use authenticated ntfy or a self-hosted instance.
- See [SECURITY.md](SECURITY.md) for vulnerability reporting.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

## License

MIT
