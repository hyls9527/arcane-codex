use tauri::State;
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use crate::utils::hash::calculate_sha256;
use tracing::{info, warn, error};

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "ico", "tiff", "tif", "avif"];

const SUPPORTED_MIME_TYPES: &[&str] = &[
    "image/jpeg", "image/png", "image/gif", "image/webp", "image/bmp", "image/x-icon",
    "image/tiff", "image/avif", "image/heic", "image/heif",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub duplicate_count: usize,
    pub error_count: usize,
    pub image_ids: Vec<i64>,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportError {
    pub file_path: String,
    pub reason: String,
}

fn validate_file(file_path: &Path) -> AppResult<(String, u64)> {
    if !file_path.exists() {
        return Err(AppError::Validation(format!("文件不存在: {}", file_path.display())));
    }

    let metadata = fs::metadata(file_path).map_err(|e| {
        AppError::Validation(format!("无法读取文件元数据: {}", e))
    })?;

    let file_size = metadata.len();
    if file_size == 0 {
        return Err(AppError::Validation(format!("文件为空: {}", file_path.display())));
    }

    if file_size > MAX_FILE_SIZE {
        return Err(AppError::Validation(format!(
            "文件大小 {} 超过限制 ({} 字节)",
            file_size, MAX_FILE_SIZE
        )));
    }

    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(AppError::Validation(format!(
            "不支持的文件格式: .{}", extension
        )));
    }

    let mime_type = match extension.as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "webp" => "image/webp".to_string(),
        "bmp" => "image/bmp".to_string(),
        "ico" => "image/x-icon".to_string(),
        "tiff" | "tif" => "image/tiff".to_string(),
        "avif" => "image/avif".to_string(),
        _ => "application/octet-stream".to_string(),
    };

    if !SUPPORTED_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(AppError::Validation(format!("不支持的 MIME 类型: {}", mime_type)));
    }

    Ok((mime_type, file_size))
}

fn is_duplicate(conn: &rusqlite::Connection, file_hash: &str) -> AppResult<bool> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM images WHERE file_hash = ?)",
        [file_hash],
        |row| row.get(0),
    ).map_err(AppError::Database)?;

    Ok(exists)
}

fn insert_image_record(
    conn: &rusqlite::Connection,
    file_path: &str,
    file_name: &str,
    file_size: u64,
    file_hash: &str,
    mime_type: &str,
) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![file_path, file_name, file_size, file_hash, mime_type],
    ).map_err(AppError::Database)?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

#[tauri::command]
pub async fn import_images(
    db: State<'_, Database>,
    file_paths: Vec<String>,
) -> AppResult<ImportResult> {
    info!("开始导入 {} 个文件", file_paths.len());

    let mut result = ImportResult {
        success_count: 0,
        duplicate_count: 0,
        error_count: 0,
        image_ids: vec![],
        errors: vec![],
    };

    let conn = db.open_connection().map_err(AppError::Database)?;

    for path_str in &file_paths {
        let file_path = Path::new(path_str);
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        match validate_file(file_path) {
            Ok((mime_type, file_size)) => {
                match calculate_sha256(file_path) {
                    Ok(hash) => {
                        match is_duplicate(&conn, &hash) {
                            Ok(true) => {
                                info!("跳过重复文件: {}", path_str);
                                result.duplicate_count += 1;
                            }
                            Ok(false) => {
                                match insert_image_record(
                                    &conn,
                                    path_str,
                                    &file_name,
                                    file_size,
                                    &hash,
                                    &mime_type,
                                ) {
                                    Ok(id) => {
                                        info!("成功导入图片: {} (ID: {})", file_name, id);
                                        result.success_count += 1;
                                        result.image_ids.push(id);
                                    }
                                    Err(e) => {
                                        error!("数据库插入失败: {} - {}", file_name, e);
                                        result.error_count += 1;
                                        result.errors.push(ImportError {
                                            file_path: path_str.clone(),
                                            reason: e.to_string(),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                error!("重复检测失败: {} - {}", file_name, e);
                                result.error_count += 1;
                                result.errors.push(ImportError {
                                    file_path: path_str.clone(),
                                    reason: e.to_string(),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        error!("哈希计算失败: {} - {}", file_name, e);
                        result.error_count += 1;
                        result.errors.push(ImportError {
                            file_path: path_str.clone(),
                            reason: format!("哈希计算失败: {}", e),
                        });
                    }
                }
            }
            Err(e) => {
                warn!("文件验证失败: {} - {}", path_str, e);
                result.error_count += 1;
                result.errors.push(ImportError {
                    file_path: path_str.clone(),
                    reason: e.to_string(),
                });
            }
        }
    }

    info!(
        "导入完成: 成功 {}, 重复 {}, 错误 {}",
        result.success_count, result.duplicate_count, result.error_count
    );

    Ok(result)
}

#[tauri::command]
pub async fn get_images(
    db: State<'_, Database>,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<serde_json::Value>> {
    let conn = db.open_connection().map_err(AppError::Database)?;

    let offset = page * page_size;

    let mut stmt = conn
        .prepare(
            "SELECT id, file_path, file_name, file_size, file_hash, mime_type,
             width, height, thumbnail_path, phash, ai_status, ai_description,
             ai_category, ai_confidence, source, created_at, updated_at
             FROM images
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(AppError::Database)?;

    let rows = stmt
        .query_map(rusqlite::params![page_size, offset], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "file_path": row.get::<_, String>(1)?,
                "file_name": row.get::<_, String>(2)?,
                "file_size": row.get::<_, i64>(3)?,
                "file_hash": row.get::<_, Option<String>>(4)?,
                "mime_type": row.get::<_, Option<String>>(5)?,
                "width": row.get::<_, Option<i32>>(6)?,
                "height": row.get::<_, Option<i32>>(7)?,
                "thumbnail_path": row.get::<_, Option<String>>(8)?,
                "phash": row.get::<_, Option<String>>(9)?,
                "ai_status": row.get::<_, String>(10)?,
                "ai_description": row.get::<_, Option<String>>(11)?,
                "ai_category": row.get::<_, Option<String>>(12)?,
                "ai_confidence": row.get::<_, Option<f64>>(13)?,
                "source": row.get::<_, String>(14)?,
                "created_at": row.get::<_, String>(15)?,
                "updated_at": row.get::<_, String>(16)?,
            }))
        })
        .map_err(AppError::Database)?;

    let images: Vec<serde_json::Value> = rows
        .filter_map(|r| match r {
            Ok(v) => Some(v),
            Err(e) => {
                error!("读取图片行失败: {}", e);
                None
            }
        })
        .collect();

    Ok(images)
}

#[tauri::command]
pub async fn get_image_detail(
    db: State<'_, Database>,
    id: i64,
) -> AppResult<serde_json::Value> {
    let conn = db.open_connection().map_err(AppError::Database)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, file_path, file_name, file_size, file_hash, mime_type,
             width, height, thumbnail_path, phash, exif_data, ai_status, ai_tags,
             ai_description, ai_category, ai_confidence, ai_model, ai_processed_at,
             ai_error_message, ai_retry_count, source, created_at, updated_at
             FROM images WHERE id = ?1",
        )
        .map_err(AppError::Database)?;

    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "file_path": row.get::<_, String>(1)?,
                "file_name": row.get::<_, String>(2)?,
                "file_size": row.get::<_, i64>(3)?,
                "file_hash": row.get::<_, Option<String>>(4)?,
                "mime_type": row.get::<_, Option<String>>(5)?,
                "width": row.get::<_, Option<i32>>(6)?,
                "height": row.get::<_, Option<i32>>(7)?,
                "thumbnail_path": row.get::<_, Option<String>>(8)?,
                "phash": row.get::<_, Option<String>>(9)?,
                "exif_data": row.get::<_, Option<String>>(10)?,
                "ai_status": row.get::<_, String>(11)?,
                "ai_tags": row.get::<_, Option<String>>(12)?,
                "ai_description": row.get::<_, Option<String>>(13)?,
                "ai_category": row.get::<_, Option<String>>(14)?,
                "ai_confidence": row.get::<_, Option<f64>>(15)?,
                "ai_model": row.get::<_, Option<String>>(16)?,
                "ai_processed_at": row.get::<_, Option<String>>(17)?,
                "ai_error_message": row.get::<_, Option<String>>(18)?,
                "ai_retry_count": row.get::<_, i32>(19)?,
                "source": row.get::<_, String>(20)?,
                "created_at": row.get::<_, String>(21)?,
                "updated_at": row.get::<_, String>(22)?,
            }))
        })
        .map_err(AppError::Database)?;

    Ok(result)
}

#[tauri::command]
pub async fn delete_images(
    db: State<'_, Database>,
    ids: Vec<i64>,
) -> AppResult<usize> {
    let conn = db.open_connection().map_err(AppError::Database)?;

    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "DELETE FROM images WHERE id IN ({})",
        placeholders.join(",")
    );

    let params: Vec<&dyn rusqlite::types::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();

    let deleted = conn
        .execute(&sql, &params[..])
        .map_err(AppError::Database)?;

    info!("删除了 {} 张图片", deleted);

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    use crate::core::db::Database;

    fn create_temp_file(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_images.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'image/jpeg', 'pending', 'import');
             INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/test/2.png', '2.png', 2000, 'hash2', 'image/png', 'completed', 'import');
             INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/test/3.jpg', '3.jpg', 3000, 'hash3', 'image/jpeg', 'completed', 'import');",
        )
        .unwrap();

        (db, temp_dir)
    }

    #[test]
    fn test_validate_file_nonexistent() {
        let path = Path::new("/nonexistent/test/image.jpg");
        let result = validate_file(path);
        assert!(result.is_err(), "不存在的文件应返回错误");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("文件不存在"));
    }

    #[test]
    fn test_validate_file_empty() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_temp_file(&temp_dir, "empty.jpg", &[]);

        let result = validate_file(&path);
        assert!(result.is_err(), "空文件应返回错误");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("文件为空"));
    }

    #[test]
    fn test_validate_file_supported_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let dummy_content = b"fake image content for testing";

        let extensions = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "ico", "tiff", "tif", "avif"];

        for ext in extensions {
            let filename = format!("test.{}", ext);
            let path = create_temp_file(&temp_dir, &filename, dummy_content);
            let result = validate_file(&path);
            assert!(result.is_ok(), "扩展名 .{} 应该被支持: {:?}", ext, result);
            let (mime_type, size) = result.unwrap();
            assert_eq!(size, dummy_content.len() as u64);
            assert!(mime_type.starts_with("image/"), "MIME 类型应该是 image/*: {}", mime_type);
        }
    }

    #[test]
    fn test_validate_file_unsupported_extension() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_temp_file(&temp_dir, "test.xyz", b"some content");

        let result = validate_file(&path);
        assert!(result.is_err(), "不支持的扩展名应返回错误");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("不支持的文件格式"));
    }

    #[test]
    fn test_validate_file_mime_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let content = b"fake image content";

        let mime_mapping = [
            ("jpg", "image/jpeg"),
            ("jpeg", "image/jpeg"),
            ("png", "image/png"),
            ("gif", "image/gif"),
            ("webp", "image/webp"),
            ("bmp", "image/bmp"),
            ("ico", "image/x-icon"),
            ("tiff", "image/tiff"),
            ("tif", "image/tiff"),
            ("avif", "image/avif"),
        ];

        for (ext, expected_mime) in mime_mapping {
            let filename = format!("test.{}", ext);
            let path = create_temp_file(&temp_dir, &filename, content);
            let result = validate_file(&path);
            assert!(result.is_ok(), "文件 {} 应该验证成功", filename);
            let (mime_type, _) = result.unwrap();
            assert_eq!(mime_type, expected_mime, "扩展名 {} 的 MIME 类型映射错误", ext);
        }
    }

    #[test]
    fn test_import_result_serialization() {
        let result = ImportResult {
            success_count: 5,
            duplicate_count: 2,
            error_count: 1,
            image_ids: vec![1, 2, 3, 4, 5],
            errors: vec![ImportError {
                file_path: "/test/error.jpg".to_string(),
                reason: "测试错误".to_string(),
            }],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ImportResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.success_count, 5);
        assert_eq!(deserialized.duplicate_count, 2);
        assert_eq!(deserialized.error_count, 1);
        assert_eq!(deserialized.image_ids.len(), 5);
        assert_eq!(deserialized.errors.len(), 1);
    }

    #[test]
    fn test_get_images_pagination() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, file_name, file_size, file_hash, mime_type, ai_status, source 
                 FROM images ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )
            .unwrap();

        let rows = stmt
            .query_map(rusqlite::params![2, 0], |row| {
                Ok((
                    row.get::<_, i64>(0).unwrap(),
                    row.get::<_, String>(1).unwrap(),
                    row.get::<_, String>(2).unwrap(),
                ))
            })
            .unwrap();

        let results: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        assert_eq!(results.len(), 2, "分页应返回 2 条记录");
    }

    #[test]
    fn test_get_images_empty_result() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT id FROM images ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )
            .unwrap();

        let rows = stmt
            .query_map(rusqlite::params![10, 100], |row| {
                Ok(row.get::<_, i64>(0).unwrap())
            })
            .unwrap();

        let results: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        assert_eq!(results.len(), 0, "超出范围应返回空结果");
    }

    #[test]
    fn test_delete_images_single() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let deleted = conn
            .execute("DELETE FROM images WHERE id = 1", [])
            .unwrap();

        assert_eq!(deleted, 1, "应删除 1 条记录");

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 2, "删除后应剩余 2 条记录");
    }

    #[test]
    fn test_delete_images_multiple() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let deleted = conn
            .execute("DELETE FROM images WHERE id IN (1, 3)", [])
            .unwrap();

        assert_eq!(deleted, 2, "应删除 2 条记录");

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 1, "删除后应剩余 1 条记录");
    }

    #[test]
    fn test_delete_images_nonexistent() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let deleted = conn
            .execute("DELETE FROM images WHERE id = 999", [])
            .unwrap();

        assert_eq!(deleted, 0, "删除不存在的记录应返回 0");
    }
}
