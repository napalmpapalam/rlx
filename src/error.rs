use eyre::Error as Report;

use crate::log::MessageBuilder;

#[derive(Debug)]
pub struct Error {
    format: bool,
    message: String,
}

impl Error {
    pub fn new(message: String) -> Self {
        Self {
            message,
            format: true,
        }
    }

    pub fn new_fmt(message: String) -> Self {
        Self {
            message: message.to_string(),
            format: false,
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = MessageBuilder::new();
        let msg = if self.format {
            msg.error(&self.message)
        } else {
            msg.error_fmt(&self.message)
        };
        write!(f, "{}", msg.build())
    }
}

impl From<Report> for Error {
    fn from(report: Report) -> Self {
        Self::new(report.to_string())
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::new(msg.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self::new(msg)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
