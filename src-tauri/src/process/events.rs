use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessOutputEvent {
    Stdout(String),
    Stderr(String),
    Finished { exit_code: Option<i32> },
    Cancelled,
    TimedOut,
}
