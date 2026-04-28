use crate::core::calibration::types::ImageCategory;
use crate::core::clip_verify::ClipClassificationResult;
use crate::utils::error::{AppResult, AppError};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSidecarConfig {
    pub python_path: String,
    pub script_path: String,
    pub model_name: String,
    pub categories: Vec<String>,
}

impl Default for ClipSidecarConfig {
    fn default() -> Self {
        Self {
            python_path: "python3".to_string(),
            script_path: "clip_sidecar.py".to_string(),
            model_name: "ViT-B/32".to_string(),
            categories: vec![
                "landscape".to_string(),
                "person".to_string(),
                "object".to_string(),
                "animal".to_string(),
                "architecture".to_string(),
                "document".to_string(),
                "other".to_string(),
            ],
        }
    }
}

pub struct ClipSidecarProcess {
    child: Child,
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<ChildStdout>,
}

impl ClipSidecarProcess {
    pub fn classify_image(&self, image_path: &str) -> AppResult<ClipClassificationResult> {
        let mut stdin = self.stdin.lock().map_err(|e| {
            AppError::validation(format!("CLIP sidecar stdin lock failed: {}", e))
        })?;

        let mut stdout = self.stdout.lock().map_err(|e| {
            AppError::validation(format!("CLIP sidecar stdout lock failed: {}", e))
        })?;

        let request = format!("{}\n", image_path);
        stdin.write_all(request.as_bytes()).map_err(|e| {
            AppError::validation(format!("CLIP sidecar write failed: {}", e))
        })?;
        stdin.flush().map_err(|e| {
            AppError::validation(format!("CLIP sidecar flush failed: {}", e))
        })?;

        let mut reader = BufReader::new(&mut *stdout);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).map_err(|e| {
            AppError::validation(format!("CLIP sidecar read failed: {}", e))
        })?;

        let response_line = response_line.trim();
        if response_line.starts_with("ERROR:") {
            return Err(AppError::ai(response_line.to_string()));
        }

        let result: ClipClassificationResult = serde_json::from_str(response_line).map_err(|e| {
            AppError::validation(format!("CLIP response parse failed: {}", e))
        })?;

        Ok(result)
    }

    pub fn health_check(&self) -> bool {
        let mut stdin = match self.stdin.lock() {
            Ok(s) => s,
            Err(_) => return false,
        };

        let mut stdout = match self.stdout.lock() {
            Ok(s) => s,
            Err(_) => return false,
        };

        if stdin.write_all(b"HEALTHCHECK\n").is_err() {
            return false;
        }
        if stdin.flush().is_err() {
            return false;
        }

        let mut reader = BufReader::new(&mut *stdout);
        let mut response = String::new();
        if reader.read_line(&mut response).is_err() {
            return false;
        }

        response.trim() == "OK"
    }
}

pub struct ClipSidecarManager;

impl ClipSidecarManager {
    pub fn start(config: &ClipSidecarConfig) -> AppResult<ClipSidecarProcess> {
        let script_path = PathBuf::from(&config.script_path);
        if !script_path.exists() {
            return Err(AppError::validation(format!(
                "CLIP sidecar script not found: {}",
                script_path.display()
            )));
        }

        let mut child = Command::new(&config.python_path)
            .arg(&config.script_path)
            .arg("--model")
            .arg(&config.model_name)
            .arg("--categories")
            .arg(serde_json::to_string(&config.categories).unwrap_or_default())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::validation(format!("CLIP sidecar start failed: {}", e)))?;

        let stdin = child.stdin.take().ok_or_else(|| {
            AppError::validation("CLIP sidecar stdin unavailable".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::validation("CLIP sidecar stdout unavailable".to_string())
        })?;

        Ok(ClipSidecarProcess {
            child,
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(stdout),
        })
    }

    pub fn stop(process: &mut ClipSidecarProcess) {
        let _ = process.child.kill();
        let _ = process.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_sidecar_config_default() {
        let config = ClipSidecarConfig::default();
        assert_eq!(config.model_name, "ViT-B/32");
        assert_eq!(config.categories.len(), 7);
        assert_eq!(config.categories[0], "landscape");
    }

    #[test]
    fn test_clip_sidecar_script_not_found() {
        let config = ClipSidecarConfig {
            script_path: "/nonexistent/clip_sidecar.py".to_string(),
            ..ClipSidecarConfig::default()
        };

        let result = ClipSidecarManager::start(&config);
        assert!(result.is_err());
    }
}
