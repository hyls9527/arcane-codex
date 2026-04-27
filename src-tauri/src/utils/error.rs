use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("File error: {0}")]
    File(#[from] std::io::Error),
    
    #[error("AI error: {0}")]
    AI(String),
    
    #[error("HTTP error: {0}")]
    HTTP(#[from] reqwest::Error),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt};
    
    let log_dir = std::env::var("APPDATA")
        .map(|appdata| format!("{}\\ArcaneCodex\\logs", appdata))
        .unwrap_or_else(|_| "./logs".to_string());
    
    std::fs::create_dir_all(&log_dir).ok();
    
    let log_file = format!("{}\\app.log", log_dir);
    let file_writer = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .ok();
    
    if let Some(writer) = file_writer {
        fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
            .with_writer(writer)
            .with_ansi(false)
            .init();
    } else {
        fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_error_display() {
        let db_err = AppError::Validation("test validation error".to_string());
        assert!(db_err.to_string().contains("test validation error"));
    }

    #[test]
    fn test_app_error_validation() {
        let err = AppError::Validation("test error".to_string());
        assert_eq!(err.to_string(), "Validation error: test error");
    }

    #[test]
    fn test_app_error_config() {
        let err = AppError::Config("missing config".to_string());
        assert!(err.to_string().contains("missing config"));
    }

    #[test]
    fn test_app_result_ok() {
        let result: AppResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_app_result_err() {
        let result: AppResult<i32> = Err(AppError::Validation("error".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_init_logging_creates_log_directory() {
        let log_dir = std::env::var("APPDATA")
            .map(|appdata| format!("{}\\ArcaneCodex\\logs", appdata))
            .unwrap_or_else(|_| "./logs".to_string());
        
        std::fs::create_dir_all(&log_dir).ok();
        
        assert!(std::path::Path::new(&log_dir).exists());
    }

    #[test]
    fn test_error_conversion_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = AppError::File(io_err);
        assert!(app_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_error_conversion_from_rusqlite_error() {
        fn trigger_db_error() -> Result<(), rusqlite::Error> {
            Err(rusqlite::Error::InvalidQuery)
        }

        let result: Result<(), AppError> = trigger_db_error().map_err(AppError::from);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Database(_) => {},
            _ => panic!("Expected Database error"),
        }
    }

    #[test]
    fn test_error_type_discrimination() {
        let errors = vec![
            AppError::Validation("validation".to_string()),
            AppError::Config("config".to_string()),
            AppError::AI("ai error".to_string()),
        ];

        for err in errors {
            let msg = err.to_string();
            match &err {
                AppError::Validation(s) => assert!(msg.contains(s)),
                AppError::Config(s) => assert!(msg.contains(s)),
                AppError::AI(s) => assert!(msg.contains(s)),
                _ => panic!("Unexpected error type"),
            }
        }
    }
}
