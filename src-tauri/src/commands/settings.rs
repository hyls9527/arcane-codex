use serde::{Serialize, Deserialize};
use tauri::State;
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub key: String,
    pub value: String,
}

#[tauri::command]
pub async fn get_config(
    db: State<'_, Database>,
    key: String,
) -> AppResult<Option<AppConfig>> {
    let conn = db.open_connection().map_err(AppError::Database)?;

    let result = conn.query_row(
        "SELECT key, value FROM app_config WHERE key = ?",
        [&key],
        |row| {
            Ok(AppConfig {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        },
    );

    match result {
        Ok(config) => {
            info!("获取配置: {} = {}", config.key, config.value);
            Ok(Some(config))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Ok(None)
        }
        Err(e) => Err(AppError::Database(e)),
    }
}

#[tauri::command]
pub async fn set_config(
    db: State<'_, Database>,
    key: String,
    value: String,
) -> AppResult<()> {
    let conn = db.open_connection().map_err(AppError::Database)?;

    conn.execute(
        "INSERT INTO app_config (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
        [&key, &value],
    ).map_err(AppError::Database)?;

    info!("保存配置: {} = {}", key, value);

    Ok(())
}

#[tauri::command]
pub async fn get_all_configs(
    db: State<'_, Database>,
) -> AppResult<Vec<AppConfig>> {
    let conn = db.open_connection().map_err(AppError::Database)?;

    let mut stmt = conn
        .prepare("SELECT key, value FROM app_config ORDER BY key")
        .map_err(AppError::Database)?;

    let configs = stmt
        .query_map([], |row| {
            Ok(AppConfig {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(AppError::Database)?;

    let result: Vec<AppConfig> = configs
        .filter_map(|r| r.ok())
        .collect();

    info!("获取所有配置: {} 条", result.len());

    Ok(result)
}

#[tauri::command]
pub async fn backup_database(output_path: String) -> AppResult<String> {
    // TODO: Implement database backup
    let _output_path = output_path;
    Ok("".to_string())
}

#[tauri::command]
pub async fn restore_database(backup_path: String) -> AppResult<()> {
    let _backup_path = backup_path;
    Ok(())
}

#[tauri::command]
pub async fn test_lm_studio_connection(url: String) -> AppResult<bool> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| AppError::Config(format!("Failed to create HTTP client: {}", e)))?;

    let health_url = format!("{}/v1/models", url.trim_end_matches('/'));

    match client.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => Ok(true),
        Ok(resp) => {
            info!("LM Studio returned status: {}", resp.status());
            Ok(false)
        }
        Err(e) => {
            info!("LM Studio connection failed: {}", e);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_settings.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig {
            key: "lm_studio_url".to_string(),
            value: "http://localhost:1234".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.key, "lm_studio_url");
        assert_eq!(deserialized.value, "http://localhost:1234");
    }

    #[test]
    fn test_set_and_get_config() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let key = "test_key".to_string();
        let value = "test_value".to_string();

        conn.execute(
            "INSERT INTO app_config (key, value) VALUES (?1, ?2)",
            [&key, &value],
        )
        .unwrap();

        let result: Option<AppConfig> = conn
            .query_row(
                "SELECT key, value FROM app_config WHERE key = ?",
                [&key],
                |row| {
                    Ok(AppConfig {
                        key: row.get(0)?,
                        value: row.get(1)?,
                    })
                },
            )
            .ok();

        assert!(result.is_some());
        let config = result.unwrap();
        assert_eq!(config.key, "test_key");
        assert_eq!(config.value, "test_value");
    }

    #[test]
    fn test_upsert_config() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let key = "custom_test_key".to_string();

        conn.execute(
            "INSERT INTO app_config (key, value) VALUES (?1, ?2)",
            [&key, "3"],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO app_config (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            [&key, "5"],
        )
        .unwrap();

        let value: String = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = ?",
                [&key],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(value, "5");
    }

    #[test]
    fn test_get_nonexistent_config() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let result: Result<AppConfig, _> = conn.query_row(
            "SELECT key, value FROM app_config WHERE key = ?",
            ["nonexistent_key"],
            |row| {
                Ok(AppConfig {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_configs() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_config", [], |row| row.get(0))
            .unwrap();

        assert!(count > 0, "初始化后应该有默认配置");
    }
}
