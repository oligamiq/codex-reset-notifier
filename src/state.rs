use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::app_server::QuotaWindow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub used_percent: f64,
    pub resets_at: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub windows: BTreeMap<u64, WindowState>,
}

impl State {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = temp_path(path);
        fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(&temp, path)?;
        Ok(())
    }

    pub fn is_reset(&self, window: &QuotaWindow, now: i64) -> bool {
        self.windows
            .get(&window.window_duration_mins)
            .is_some_and(|prev| {
                let recovered = prev.used_percent >= 99.9 && window.used_percent < 99.9;
                let has_old_deadline = prev.resets_at > 0;
                let schedule_advanced = window.resets_at > prev.resets_at + 30;
                let old_reset_was_due = now >= prev.resets_at.saturating_sub(180);
                recovered || (has_old_deadline && schedule_advanced && old_reset_was_due)
            })
    }

    pub fn update(&mut self, window: &QuotaWindow) {
        self.windows.insert(
            window.window_duration_mins,
            WindowState {
                used_percent: window.used_percent,
                resets_at: window.resets_at,
            },
        );
    }
}

fn temp_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_owned();
    os.push(".tmp");
    PathBuf::from(os)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(used: f64, resets_at: i64) -> QuotaWindow {
        QuotaWindow {
            used_percent: used,
            window_duration_mins: 300,
            resets_at,
        }
    }

    #[test]
    fn first_observation_is_not_a_reset() {
        let state = State::default();
        assert!(!state.is_reset(&window(100.0, 1_000), 900));
    }

    #[test]
    fn exhaustion_recovery_is_a_reset() {
        let mut state = State::default();
        state.update(&window(100.0, 1_000));
        assert!(state.is_reset(&window(0.0, 2_000), 1_001));
    }

    #[test]
    fn due_schedule_advance_is_a_reset() {
        let mut state = State::default();
        state.update(&window(50.0, 1_000));
        assert!(state.is_reset(&window(0.0, 2_000), 1_000));
    }

    #[test]
    fn early_schedule_change_is_not_a_reset() {
        let mut state = State::default();
        state.update(&window(50.0, 1_000));
        assert!(!state.is_reset(&window(40.0, 2_000), 700));
    }
}

#[cfg(test)]
mod zero_deadline_test {
    use super::*;

    #[test]
    fn zero_deadline_does_not_fake_a_reset() {
        let mut state = State::default();
        state.windows.insert(
            300,
            WindowState {
                used_percent: 10.0,
                resets_at: 0,
            },
        );
        let current = QuotaWindow {
            used_percent: 10.0,
            window_duration_mins: 300,
            resets_at: 2_000,
        };
        assert!(!state.is_reset(&current, 2_000));
    }
}
