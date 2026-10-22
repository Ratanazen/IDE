use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum CancellationError {
    #[error("Failed to signal process group {0}: {1}")]
    SignalFailed(i32, nix::Error),
    #[error("Invalid process group ID: {0}")]
    InvalidPgid(i32),
}

/// Terminates an entire process group by sending SIGTERM, waiting for a grace period,
/// and escalating to SIGKILL if any processes remain alive.
pub async fn terminate_process_group(pgid: i32, grace_period: Duration) -> Result<(), CancellationError> {
    if pgid <= 1 {
        return Err(CancellationError::InvalidPgid(pgid));
    }

    let pid = Pid::from_raw(pgid);

    match killpg(pid, Signal::SIGTERM) {
        Ok(_) => {}
        Err(nix::errno::Errno::ESRCH) => {
            return Ok(());
        }
        Err(e) => {
            return Err(CancellationError::SignalFailed(pgid, e));
        }
    }

    sleep(grace_period).await;

    match killpg(pid, None) {
        Ok(_) => {
            let _ = killpg(pid, Signal::SIGKILL);
        }
        Err(nix::errno::Errno::ESRCH) => {
            return Ok(());
        }
        Err(_) => {}
    }

    Ok(())
}
