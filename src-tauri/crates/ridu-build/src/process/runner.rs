use super::cancellation::terminate_process_group;
use ridu_core::models::ProcessOutputEvent;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Failed to spawn process {0:?}: {1}")]
    SpawnFailed(PathBuf, std::io::Error),
    #[error("Process timed out after {0:?}")]
    TimedOut(Duration),
    #[error("Process was cancelled")]
    Cancelled,
    #[error("I/O error during execution: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub timeout: Option<Duration>,
}

impl CommandSpec {
    pub fn new<P: Into<PathBuf>>(executable: P) -> Self {
        Self {
            executable: executable.into(),
            args: Vec::new(),
            cwd: None,
            env: HashMap::new(),
            timeout: None,
        }
    }

    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self.args.push(arg.into());
        }
        self
    }

    pub fn current_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.cwd = Some(dir.into());
        self
    }

    pub fn env<K: Into<String>, V: Into<String>>(mut self, key: K, val: V) -> Self {
        self.env.insert(key.into(), val.into());
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ProcessExecutionResult {
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub cancelled: bool,
}

#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

pub struct ProcessRunner;

impl ProcessRunner {
    /// Spawns and executes a command, streaming stdout and stderr asynchronously to the given channel.
    /// Manages process groups so cancellation terminates all child processes.
    pub async fn run(
        spec: CommandSpec,
        event_sender: mpsc::Sender<ProcessOutputEvent>,
        cancellation: CancellationToken,
    ) -> Result<ProcessExecutionResult, ProcessError> {
        let mut cmd = Command::new(&spec.executable);
        cmd.args(&spec.args);

        if let Some(ref cwd) = spec.cwd {
            cmd.current_dir(cwd);
        }

        for (k, v) in &spec.env {
            cmd.env(k, v);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                nix::unistd::setpgid(nix::unistd::Pid::from_raw(0), nix::unistd::Pid::from_raw(0))
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            });
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| ProcessError::SpawnFailed(spec.executable.clone(), e))?;

        let pid = child.id().unwrap_or(0) as i32;
        let pgid = pid;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let tx_out = event_sender.clone();
        let stdout_task = tokio::spawn(async move {
            if let Some(out) = stdout {
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = tx_out.send(ProcessOutputEvent::Stdout(line)).await;
                }
            }
        });

        let tx_err = event_sender.clone();
        let stderr_task = tokio::spawn(async move {
            if let Some(err) = stderr {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = tx_err.send(ProcessOutputEvent::Stderr(line)).await;
                }
            }
        });

        let wait_for_completion = async {
            loop {
                if cancellation.is_cancelled() {
                    let _ = terminate_process_group(pgid, Duration::from_millis(1000)).await;
                    let _ = event_sender.send(ProcessOutputEvent::Cancelled).await;
                    return Ok::<ProcessExecutionResult, ProcessError>(ProcessExecutionResult {
                        exit_code: None,
                        timed_out: false,
                        cancelled: true,
                    });
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        let _ = stdout_task.await;
                        let _ = stderr_task.await;
                        let code = status.code();
                        let _ = event_sender
                            .send(ProcessOutputEvent::Finished { exit_code: code })
                            .await;
                        return Ok(ProcessExecutionResult {
                            exit_code: code,
                            timed_out: false,
                            cancelled: false,
                        });
                    }
                    Ok(None) => {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    Err(e) => {
                        return Err(ProcessError::Io(e));
                    }
                }
            }
        };

        if let Some(limit) = spec.timeout {
            match timeout(limit, wait_for_completion).await {
                Ok(res) => res,
                Err(_) => {
                    let _ = terminate_process_group(pgid, Duration::from_millis(500)).await;
                    let _ = event_sender.send(ProcessOutputEvent::TimedOut).await;
                    Ok(ProcessExecutionResult {
                        exit_code: None,
                        timed_out: true,
                        cancelled: false,
                    })
                }
            }
        } else {
            wait_for_completion.await
        }
    }
}
