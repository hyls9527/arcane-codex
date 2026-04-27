use tauri::State;
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct BatchUpdateTagsRequest {
    pub image_ids: Vec<i64>,
    pub tags: Vec<String>,
    pub mode: TagUpdateMode,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum TagUpdateMode {
    Add,
    Remove,
    Replace,
}

#[derive(Debug, Deserialize)]
pub struct BatchUpdateCategoryRequest {
    pub image_ids: Vec<i64>,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchUpdateFieldsRequest {
    pub image_ids: Vec<i64>,
    pub updates: Vec<FieldUpdate>,
}

#[derive(Debug, Deserialize)]
pub struct FieldUpdate {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct BatchResult {
    pub updated_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn batch_update_tags(
    db: State<'_, Database>,
    request: BatchUpdateTagsRequest,
) -> AppResult<BatchResult> {
    if request.image_ids.is_empty() {
        return Err(AppError::Validation("请选择至少一张图片".to_string()));
    }

    if request.tags.is_empty() && request.mode != TagUpdateMode::Remove {
        return Err(AppError::Validation("标签不能为空".to_string()));
    }

    let conn = db.open_connection()?;
    let mut updated_count = 0usize;
    let mut failed_count = 0usize;
    let mut errors = Vec::new();

    for &image_id in &request.image_ids {
        match update_single_image_tags(&conn, image_id, &request.tags, &request.mode) {
            Ok(_) => updated_count += 1,
            Err(e) => {
                failed_count += 1;
                errors.push(format!("图片 {} 更新失败: {}", image_id, e));
            }
        }
    }

    info!(
        "批量标签更新完成: 成功 {}, 失败 {}",
        updated_count, failed_count
    );

    Ok(BatchResult {
        updated_count,
        failed_count,
        errors,
    })
}

#[tauri::command]
pub async fn batch_update_category(
    db: State<'_, Database>,
    request: BatchUpdateCategoryRequest,
) -> AppResult<BatchResult> {
    if request.image_ids.is_empty() {
        return Err(AppError::Validation("请选择至少一张图片".to_string()));
    }

    if request.category.trim().is_empty() {
        return Err(AppError::Validation("分类不能为空".to_string()));
    }

    let mut conn = db.open_connection()?;
    let mut updated_count = 0usize;
    let mut failed_count = 0usize;
    let mut errors = Vec::new();

    let tx = conn.transaction().map_err(AppError::Database)?;

    {
        let mut stmt = tx
            .prepare(
                "UPDATE images SET ai_category = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .map_err(AppError::Database)?;

        for &image_id in &request.image_ids {
            match stmt.execute(rusqlite::params![request.category, image_id]) {
                Ok(_) => updated_count += 1,
                Err(e) => {
                    failed_count += 1;
                    errors.push(format!("图片 {} 更新失败: {}", image_id, e));
                }
            }
        }
    }

    tx.commit().map_err(AppError::Database)?;

    info!(
        "批量分类更新完成: 成功 {}, 失败 {}",
        updated_count, failed_count
    );

    Ok(BatchResult {
        updated_count,
        failed_count,
        errors,
    })
}

#[tauri::command]
pub async fn batch_update_fields(
    db: State<'_, Database>,
    request: BatchUpdateFieldsRequest,
) -> AppResult<BatchResult> {
    if request.image_ids.is_empty() {
        return Err(AppError::Validation("请选择至少一张图片".to_string()));
    }

    if request.updates.is_empty() {
        return Err(AppError::Validation("更新字段不能为空".to_string()));
    }

    for update in &request.updates {
        if !is_valid_field(&update.field) {
            return Err(AppError::Validation(format!(
                "不支持的字段: {}", update.field
            )));
        }
    }

    let mut conn = db.open_connection()?;
    let mut updated_count = 0usize;
    let mut failed_count = 0usize;
    let mut errors = Vec::new();

    for &image_id in &request.image_ids {
        let tx = conn.transaction();
        if tx.is_err() {
            failed_count += 1;
            errors.push(format!("图片 {} 事务创建失败", image_id));
            continue;
        }
        let tx = tx.unwrap();

        let mut success = true;
        for update in &request.updates {
            let sql = format!(
                "UPDATE images SET {} = ?, updated_at = datetime('now') WHERE id = ?",
                update.field
            );

            match tx.execute(&sql, rusqlite::params![update.value, image_id]) {
                Ok(_) => {}
                Err(e) => {
                    success = false;
                    errors.push(format!("图片 {} 字段 {} 更新失败: {}", image_id, update.field, e));
                    break;
                }
            }
        }

        if success {
            if tx.commit().is_err() {
                failed_count += 1;
                errors.push(format!("图片 {} 事务提交失败", image_id));
            } else {
                updated_count += 1;
            }
        } else {
            failed_count += 1;
            let _ = tx.rollback();
        }
    }

    info!(
        "批量字段更新完成: 成功 {}, 失败 {}",
        updated_count, failed_count
    );

    Ok(BatchResult {
        updated_count,
        failed_count,
        errors,
    })
}

fn update_single_image_tags(
    conn: &rusqlite::Connection,
    image_id: i64,
    new_tags: &[String],
    mode: &TagUpdateMode,
) -> AppResult<()> {
    let current_tags: Option<String> = conn
        .query_row(
            "SELECT ai_tags FROM images WHERE id = ?",
            rusqlite::params![image_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let mut tags: Vec<String> = current_tags
        .map(|s| {
            serde_json::from_str(&s)
                .unwrap_or_default()
        })
        .unwrap_or_default();

    match mode {
        TagUpdateMode::Add => {
            for tag in new_tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
        }
        TagUpdateMode::Remove => {
            tags.retain(|t| !new_tags.contains(t));
        }
        TagUpdateMode::Replace => {
            tags = new_tags.to_vec();
        }
    }

    let tags_json = serde_json::to_string(&tags).unwrap_or("[]".to_string());

    conn.execute(
        "UPDATE images SET ai_tags = ?, updated_at = datetime('now') WHERE id = ?",
        rusqlite::params![tags_json, image_id],
    )
    .map_err(AppError::Database)?;

    Ok(())
}

fn is_valid_field(field: &str) -> bool {
    matches!(
        field,
        "ai_description" | "ai_category" | "ai_confidence" | "source" | "file_name"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_batch.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();

        let conn = db.open_connection().unwrap();
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_category) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'completed', '[\"cat\",\"animal\"]', 'animal');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_category) 
             VALUES ('/test/2.jpg', '2.jpg', 2000, 'hash2', 'completed', '[\"dog\",\"pet\"]', 'animal');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_category) 
             VALUES ('/test/3.jpg', '3.jpg', 3000, 'hash3', 'completed', NULL, 'landscape');",
        )
        .unwrap();

        (db, temp_dir)
    }

    #[test]
    fn test_batch_update_tags_add_mode() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let result = update_single_image_tags(
            &conn,
            1,
            &["cute".to_string(), "pet".to_string()],
            &TagUpdateMode::Add,
        );
        assert!(result.is_ok());

        let tags: String = conn
            .query_row(
                "SELECT ai_tags FROM images WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let parsed: Vec<String> = serde_json::from_str(&tags).unwrap();
        assert!(parsed.contains(&"cat".to_string()));
        assert!(parsed.contains(&"animal".to_string()));
        assert!(parsed.contains(&"cute".to_string()));
        assert!(parsed.contains(&"pet".to_string()));
    }

    #[test]
    fn test_batch_update_tags_remove_mode() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let result = update_single_image_tags(
            &conn,
            1,
            &["cat".to_string()],
            &TagUpdateMode::Remove,
        );
        assert!(result.is_ok());

        let tags: String = conn
            .query_row(
                "SELECT ai_tags FROM images WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let parsed: Vec<String> = serde_json::from_str(&tags).unwrap();
        assert!(!parsed.contains(&"cat".to_string()));
        assert!(parsed.contains(&"animal".to_string()));
    }

    #[test]
    fn test_batch_update_tags_replace_mode() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let result = update_single_image_tags(
            &conn,
            1,
            &["new_tag_1".to_string(), "new_tag_2".to_string()],
            &TagUpdateMode::Replace,
        );
        assert!(result.is_ok());

        let tags: String = conn
            .query_row(
                "SELECT ai_tags FROM images WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let parsed: Vec<String> = serde_json::from_str(&tags).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.contains(&"new_tag_1".to_string()));
        assert!(parsed.contains(&"new_tag_2".to_string()));
    }

    #[test]
    fn test_batch_update_tags_empty_image() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let result = update_single_image_tags(
            &conn,
            3,
            &["new_tag".to_string()],
            &TagUpdateMode::Add,
        );
        assert!(result.is_ok());

        let tags: Option<String> = conn
            .query_row(
                "SELECT ai_tags FROM images WHERE id = 3",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let parsed: Vec<String> = serde_json::from_str(&tags.unwrap()).unwrap();
        assert_eq!(parsed, vec!["new_tag"]);
    }

    #[test]
    fn test_batch_update_category() {
        let (db, _temp) = setup_test_db();
        let mut conn = db.open_connection().unwrap();

        let tx = conn.transaction().unwrap();
        {
            let mut stmt = tx
                .prepare(
                    "UPDATE images SET ai_category = ?, updated_at = datetime('now') WHERE id = ?",
                )
                .unwrap();

            for &id in &[1i64, 2, 3] {
                stmt.execute(rusqlite::params!["portrait", id]).unwrap();
            }
        }
        tx.commit().unwrap();

        let verify_conn = db.open_connection().unwrap();
        for &id in &[1i64, 2, 3] {
            let category: String = verify_conn
                .query_row(
                    "SELECT ai_category FROM images WHERE id = ?",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(category, "portrait");
        }
    }

    #[test]
    fn test_is_valid_field() {
        assert!(is_valid_field("ai_description"));
        assert!(is_valid_field("ai_category"));
        assert!(is_valid_field("ai_confidence"));
        assert!(is_valid_field("source"));
        assert!(is_valid_field("file_name"));
        assert!(!is_valid_field("ai_tags"));
        assert!(!is_valid_field("file_path"));
        assert!(!is_valid_field("invalid_field"));
    }

    #[test]
    fn test_batch_result_structure() {
        let result = BatchResult {
            updated_count: 5,
            failed_count: 2,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        assert_eq!(result.updated_count, 5);
        assert_eq!(result.failed_count, 2);
        assert_eq!(result.errors.len(), 2);
    }
}
