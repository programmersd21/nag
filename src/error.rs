use std::fmt;

#[derive(Debug)]
pub enum NagError {
    NoCommandSpecified,
    CommandNotFound {
        command: String,
        source: std::io::Error,
    },
    CommandSpawnFailed {
        command: String,
        source: std::io::Error,
    },
    InvalidDuration {
        input: String,
        reason: String,
    },
}

impl fmt::Display for NagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCommandSpecified => write!(f, "nag: no command specified"),
            Self::CommandNotFound { command, source } => {
                write!(f, "nag: could not start `{}`: {}", command, source)
            }
            Self::CommandSpawnFailed { command, source } => {
                write!(f, "nag: failed to execute `{}`: {}", command, source)
            }
            Self::InvalidDuration { input, reason } => {
                write!(f, "nag: invalid duration `{}`: {}", input, reason)
            }
        }
    }
}

impl std::error::Error for NagError {}
