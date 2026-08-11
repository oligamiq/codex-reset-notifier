mod app_server;
mod notifier;
mod state;

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use clap::Parser;

use app_server::{AppServer, QuotaWindow};
use notifier::NtfyNotifier;
use state::State;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, env = "CODEX_NOTIFY_NTFY_TOPIC")]
    ntfy_topic: Option<String>,
    #[arg(
        long,
        env = "CODEX_NOTIFY_NTFY_SERVER",
        default_value = "https://ntfy.sh"
    )]
    ntfy_server: String,
    #[arg(long, env = "CODEX_NOTIFY_NTFY_TOKEN")]
    ntfy_token: Option<String>,
    #[arg(long, env = "CODEX_NOTIFY_INTERVAL_SECS", default_value_t = 60)]
    interval_secs: u64,
    #[arg(long, env = "CODEX_NOTIFY_STATE_FILE")]
    state_file: Option<PathBuf>,
    #[arg(long, default_value = "codex")]
    codex_command: String,
    #[arg(long)]
    once: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    test_notification: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.interval_secs < 10 {
        bail!("--interval-secs must be at least 10");
    }

    let state_path = cli.state_file.clone().unwrap_or_else(default_state_path);
    let mut state = State::load(&state_path)?;
    let notifier = match (&cli.ntfy_topic, cli.dry_run) {
        (Some(topic), _) => Some(NtfyNotifier::new(
            &cli.ntfy_server,
            topic,
            cli.ntfy_token.clone(),
        )),
        (None, true) => None,
        (None, false) => bail!("set --ntfy-topic or CODEX_NOTIFY_NTFY_TOPIC"),
    };

    if cli.test_notification {
        if cli.dry_run {
            println!("DRY RUN: Codex reset notifier test");
        } else {
            notifier.as_ref().expect("validated above").send(
                "Codex reset notifier test",
                "Push notifications are configured correctly.",
            )?;
            println!("test notification sent");
        }
        return Ok(());
    }

    loop {
        match AppServer::start(&cli.codex_command) {
            Ok(mut server) => loop {
                match poll_once(
                    &mut server,
                    &mut state,
                    &state_path,
                    notifier.as_ref(),
                    cli.dry_run,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        eprintln!("poll failed: {err:#}");
                        if cli.once {
                            return Err(err);
                        }
                        break;
                    }
                }
                if cli.once {
                    return Ok(());
                }
                thread::sleep(Duration::from_secs(cli.interval_secs));
            },
            Err(err) => {
                eprintln!("failed to start Codex app-server: {err:#}");
                if cli.once {
                    return Err(err);
                }
            }
        }
        thread::sleep(Duration::from_secs(cli.interval_secs.min(30)));
    }
}

fn poll_once(
    server: &mut AppServer,
    state: &mut State,
    state_path: &std::path::Path,
    notifier: Option<&NtfyNotifier>,
    dry_run: bool,
) -> Result<()> {
    let windows = server.read_quota_windows()?;
    let now = unix_now();
    for window in &windows {
        println!(
            "{}: used={:.1}% remaining={:.1}% resets_at={}",
            window_label(window),
            window.used_percent,
            (100.0 - window.used_percent).max(0.0),
            window.resets_at,
        );

        if state.is_reset(window, now) {
            let title = format!("Codex {} quota reset", window_label(window));
            let body = format!(
                "Quota is available again. Usage: {:.1}% ({:.1}% remaining).",
                window.used_percent,
                (100.0 - window.used_percent).max(0.0),
            );
            if dry_run {
                println!("DRY RUN notification: {title} — {body}");
            } else if let Some(notifier) = notifier {
                notifier.send(&title, &body)?;
                println!("notification sent: {title}");
            }
        }

        state.update(window);
        state.save(state_path)?;
    }
    Ok(())
}

fn window_label(window: &QuotaWindow) -> String {
    match window.window_duration_mins {
        300 => "5-hour".to_owned(),
        10_080 => "weekly".to_owned(),
        mins if mins % 1_440 == 0 => format!("{}-day", mins / 1_440),
        mins if mins % 60 == 0 => format!("{}-hour", mins / 60),
        mins => format!("{mins}-minute"),
    }
}

fn default_state_path() -> PathBuf {
    if let Some(dir) = env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(dir).join("codex-reset-notifier/state.json");
    }
    let home = env::var_os("HOME").expect("HOME is not set");
    PathBuf::from(home).join(".local/state/codex-reset-notifier/state.json")
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
