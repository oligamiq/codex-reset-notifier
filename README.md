# codex-reset-notifier

A small Rust daemon that sends a smartphone push notification when a Codex ChatGPT quota window resets.

It does **not** read or store OpenAI tokens itself. It starts the installed `codex app-server`, uses Codex's existing login, and polls the supported `account/rateLimits/read` method.

## Notification path

`Codex app-server -> codex-reset-notifier -> ntfy -> Android/iOS`

The watcher supports whatever Codex quota windows the server returns. It does not assume that both the 5-hour and weekly windows are always present.

## Build

```bash
cargo build --release
```

Check the currently visible quota without sending anything:

```bash
cargo run -- --dry-run --once
```

## Configure ntfy

Install the ntfy app on the phone and subscribe to a long, random topic. Then set:

```bash
export CODEX_NOTIFY_NTFY_TOPIC='your-random-topic'
# Optional for authenticated/self-hosted ntfy:
# export CODEX_NOTIFY_NTFY_SERVER='https://ntfy.example.com'
# export CODEX_NOTIFY_NTFY_TOKEN='tk_...'
```

Test push delivery:

```bash
cargo run -- --test-notification
```

Run continuously:

```bash
cargo run --release --
```

State is persisted under `$XDG_STATE_HOME/codex-reset-notifier/state.json`, or `~/.local/state/codex-reset-notifier/state.json` when `XDG_STATE_HOME` is unset. This lets a restart still notice that a stored reset deadline has passed.

## systemd user service

After building, copy the binary and service file:

```bash
install -Dm755 target/release/codex-reset-notifier ~/.local/bin/codex-reset-notifier
mkdir -p ~/.config/codex-reset-notifier ~/.config/systemd/user
cp .env.example ~/.config/codex-reset-notifier/env
cp packaging/systemd/codex-reset-notifier.service ~/.config/systemd/user/
```

Edit `~/.config/codex-reset-notifier/env`, then:

```bash
systemctl --user daemon-reload
systemctl --user enable --now codex-reset-notifier.service
journalctl --user -u codex-reset-notifier.service -f
```

## Reset detection

A reset is emitted when a previously observed quota window becomes available after exhaustion, or when its next-reset timestamp advances after the old deadline becomes due. The state is updated only after notification delivery succeeds, so a transient ntfy failure is retried on the next poll.
