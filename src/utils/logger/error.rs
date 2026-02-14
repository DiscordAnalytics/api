use std::{error::Error, fmt};

#[derive(Debug)]
pub enum LoggerError {
    Initialization(String),
    FileCreation(String),
}

impl fmt::Display for LoggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoggerError::Initialization(msg) => write!(f, "Logger initialization failed: {}", msg),
            LoggerError::FileCreation(msg) => write!(f, "Log file creation failed: {}", msg),
        }
    }
}

impl Error for LoggerError {}
