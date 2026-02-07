use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum DetectorEvent {
    TouchStarted,
    TouchCompleted,
}

enum State {
    Idle,
    MaybePending(Instant),
    TouchPending,
}

pub struct Detector {
    poll_interval: Duration,
    grace_period: Duration,
    shutdown: Arc<AtomicBool>,
}

impl Detector {
    pub fn new(shutdown: Arc<AtomicBool>) -> Self {
        Self {
            poll_interval: Duration::from_millis(300),
            grace_period: Duration::from_millis(500),
            shutdown,
        }
    }

    pub fn run(&mut self, mut on_event: impl FnMut(DetectorEvent)) {
        let mut state = State::Idle;

        log::info!("watching for ssh-keygen signing processes");

        while !self.shutdown.load(Ordering::Relaxed) {
            let signing = Self::is_signing_in_progress();

            match (&state, signing) {
                (State::Idle, true) => {
                    log::debug!("signing process detected");
                    state = State::MaybePending(Instant::now());
                }
                (State::MaybePending(since), true) => {
                    if since.elapsed() >= self.grace_period {
                        log::info!("yubikey touch needed");
                        state = State::TouchPending;
                        on_event(DetectorEvent::TouchStarted);
                    }
                }
                (State::MaybePending(_), false) => {
                    log::debug!("signing completed quickly, no touch needed");
                    state = State::Idle;
                }
                (State::TouchPending, false) => {
                    log::info!("touch completed");
                    state = State::Idle;
                    on_event(DetectorEvent::TouchCompleted);
                }
                _ => {}
            }

            std::thread::sleep(self.poll_interval);
        }

        if matches!(state, State::TouchPending) {
            on_event(DetectorEvent::TouchCompleted);
        }
    }

    fn is_signing_in_progress() -> bool {
        Command::new("pgrep")
            .args(["-f", "ssh-keygen.*-Y sign"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }
}
