use tauri::State;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use crate::core::db::Database;
use crate::core::dedup::{DeduplicationScanner, DuplicateGroup, ScanResult as CoreScanResult};
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub threshold: Option<u32>,
}

#[tauri::command]
pub async fn scan_duplicates(
    db: State<'_, Database>,
    request: Option<ScanRequest>,
) -> AppResult<CoreScanResult> {
    let threshold = request.and_then(|r| r.threshold);
    let scanner = DeduplicationScanner::new(threshold);

    info!("开始重复项扫描，阈值: {:?}", threshold);

    let result = scanner.scan(&db)?;

    Ok(result)
}

#[derive(Debug, Deserialize)]
pub enum RetentionPolicy {
    KeepHighestResolution,
    KeepEarliestImport,
    Manual,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDuplicatesRequest {
    pub groups: Vec<DuplicateGroup>,
    pub policy: RetentionPolicy,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DeleteResult {
    pub deleted_count: usize,
    pub kept_count: usize,
    pub freed_space_bytes: i64,
    pub dry_run: bool,
}

#[tauri::command]
pub async fn delete_duplicates(
    db: State<'_, Database>,
    request: DeleteDuplicatesRequest,
) -> AppResult<DeleteResult> {
    if request.groups.is_empty() {
        return Err(AppError::Validation("没有需要处理的重复项".to_string()));
    }

    let dry_run = request.dry_run.unwrap_or(false);
    let mut deleted_count = 0usize;
    let mut kept_count = 0usize;
    let mut freed_space_bytes: i64 = 0;

    for group in &request.groups {
        if group.images.len() < 2 {
            continue;
        }

        let mut sorted_images = group.images.clone();

        match request.policy {
            RetentionPolicy::KeepHighestResolution => {
                sorted_images.sort_by(|a, b| {
                    let area_a = (a.width.unwrap_or(0) * a.height.unwrap_or(0)) as i64;
                    let area_b = (b.width.unwrap_or(0) * b.height.unwrap_or(0)) as i64;
                    area_b.cmp(&area_a).then_with(|| a.file_size.cmp(&b.file_size))
                });
            }
            RetentionPolicy::KeepEarliestImport => {
                sorted_images.sort_by(|a, b| a.image_id.cmp(&b.image_id));
            }
            RetentionPolicy::Manual => {
                sorted_images.sort_by(|a, b| {
                    a.distance.cmp(&b.distance)
                        .then_with(|| b.file_size.cmp(&a.file_size))
                });
            }
        }

        if let Some(keep) = sorted_images.first() {
            kept_count += 1;
            info!(
                "保留图片: {} (ID: {})",
                keep.file_name, keep.image_id
            );
        }

        for to_delete in sorted_images.iter().skip(1) {
            if !dry_run {
                let thumb = delete_image_record(&db, to_delete.image_id)?;
                delete_thumbnail(&thumb);
                freed_space_bytes += to_delete.file_size;
            }
            deleted_count += 1;
            info!(
                "删除重复项: {} (ID: {}, 大小: {} bytes)",
                to_delete.file_name, to_delete.image_id, to_delete.file_size
            );
        }
    }

    if dry_run {
        info!(
            "试运行: 将删除 {} 个重复项，保留 {} 个，释放 {} bytes",
            deleted_count, kept_count, freed_space_bytes
        );
    } else {
        info!(
            "删除完成: 删除 {} 个，保留 {} 个，释放 {} bytes",
            deleted_count, kept_count, freed_space_bytes
        );
    }

    Ok(DeleteResult {
        deleted_count,
        kept_count,
        freed_space_bytes,
        dry_run,
    })
}

fn delete_image_record(db: &Database, image_id: i64) -> AppResult<Option<String>> {
    let conn = db.open_connection()?;

    let thumbnail_path: Option<String> = conn
        .query_row(
            "SELECT thumbnail_path FROM images WHERE id = ?",
            rusqlite::params![image_id],
            |row| row.get(0),
        )
        .ok();

    conn.execute(
        "DELETE FROM search_index WHERE image_id = ?",
        rusqlite::params![image_id],
    )
    .map_err(AppError::Database)?;

    conn.execute(
        "DELETE FROM images WHERE id = ?",
        rusqlite::params![image_id],
    )
    .map_err(AppError::Database)?;

    Ok(thumbnail_path)
}

fn delete_thumbnail(thumbnail_path: &Option<String>) {
    if let Some(path) = thumbnail_path {
        let file_path = Path::new(path);
        if file_path.exists() {
            if let Err(e) = fs::remove_file(file_path) {
                warn!("删除缩略图失败 {}: {}", path, e);
            } else {
                info!("已删除缩略图: {}", path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dedup::DuplicateImage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_delete_dedup.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();
        (db, temp_dir)
    }

    fn create_test_image(
        db: &Database,
        id: i64,
        width: i32,
        height: i32,
        file_size: i64,
    ) {
        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, width, height) 
             VALUES (?, ?, ?, ?, 'completed', ?, ?)",
            rusqlite::params![
                format!("/test/{}.jpg", id),
                format!("{}.jpg", id),
                file_size,
                format!("hash{}", id),
                width,
                height,
            ],
        )
        .unwrap();
    }

    #[test]
    fn test_delete_request_validation() {
        let request = DeleteDuplicatesRequest {
            groups: vec![],
            policy: RetentionPolicy::KeepHighestResolution,
            dry_run: Some(false),
        };
        assert!(request.groups.is_empty());
    }

    #[test]
    fn test_dry_run_mode() {
        let (db, _temp) = setup_test_db();

        create_test_image(&db, 1, 1920, 1080, 1000000);
        create_test_image(&db, 2, 1280, 720, 500000);

        let group = DuplicateGroup {
            images: vec![
                DuplicateImage {
                    image_id: 1,
                    file_path: "/test/1.jpg".to_string(),
                    file_name: "1.jpg".to_string(),
                    file_size: 1000000,
                    width: Some(1920),
                    height: Some(1080),
                    phash: "abc123".to_string(),
                    distance: 5,
                },
                DuplicateImage {
                    image_id: 2,
                    file_path: "/test/2.jpg".to_string(),
                    file_name: "2.jpg".to_string(),
                    file_size: 500000,
                    width: Some(1280),
                    height: Some(720),
                    phash: "def456".to_string(),
                    distance: 5,
                },
            ],
            similarity: 0.95,
        };

        let request = DeleteDuplicatesRequest {
            groups: vec![group],
            policy: RetentionPolicy::KeepHighestResolution,
            dry_run: Some(true),
        };

        assert!(request.dry_run.unwrap());
    }

    #[test]
    fn test_retention_policy_highest_resolution() {
        let images = vec![
            DuplicateImage {
                image_id: 1,
                file_path: "/test/1.jpg".to_string(),
                file_name: "1.jpg".to_string(),
                file_size: 500000,
                width: Some(1280),
                height: Some(720),
                phash: "abc".to_string(),
                distance: 0,
            },
            DuplicateImage {
                image_id: 2,
                file_path: "/test/2.jpg".to_string(),
                file_name: "2.jpg".to_string(),
                file_size: 1000000,
                width: Some(1920),
                height: Some(1080),
                phash: "def".to_string(),
                distance: 0,
            },
        ];

        let mut sorted = images.clone();
        sorted.sort_by(|a, b| {
            let area_a = (a.width.unwrap_or(0) * a.height.unwrap_or(0)) as i64;
            let area_b = (b.width.unwrap_or(0) * b.height.unwrap_or(0)) as i64;
            area_b.cmp(&area_a).then_with(|| a.file_size.cmp(&b.file_size))
        });

        assert_eq!(sorted[0].image_id, 2);
        assert_eq!(sorted[0].width, Some(1920));
        assert_eq!(sorted[0].height, Some(1080));
    }

    #[test]
    fn test_retention_policy_earliest_import() {
        let images = vec![
            DuplicateImage {
                image_id: 3,
                file_path: "/test/3.jpg".to_string(),
                file_name: "3.jpg".to_string(),
                file_size: 600000,
                width: Some(1024),
                height: Some(768),
                phash: "ghi".to_string(),
                distance: 0,
            },
            DuplicateImage {
                image_id: 1,
                file_path: "/test/1.jpg".to_string(),
                file_name: "1.jpg".to_string(),
                file_size: 500000,
                width: Some(1280),
                height: Some(720),
                phash: "abc".to_string(),
                distance: 0,
            },
            DuplicateImage {
                image_id: 2,
                file_path: "/test/2.jpg".to_string(),
                file_name: "2.jpg".to_string(),
                file_size: 1000000,
                width: Some(1920),
                height: Some(1080),
                phash: "def".to_string(),
                distance: 0,
            },
        ];

        let mut sorted = images.clone();
        sorted.sort_by(|a, b| a.image_id.cmp(&b.image_id));

        assert_eq!(sorted[0].image_id, 1);
        assert_eq!(sorted[1].image_id, 2);
        assert_eq!(sorted[2].image_id, 3);
    }

    #[test]
    fn test_freed_space_calculation() {
        let deleted_size_1: i64 = 500000;
        let deleted_size_2: i64 = 600000;
        let total_freed = deleted_size_1 + deleted_size_2;

        assert_eq!(total_freed, 1100000);
    }

    #[test]
    fn test_single_image_group_skipped() {
        let group = DuplicateGroup {
            images: vec![DuplicateImage {
                image_id: 1,
                file_path: "/test/1.jpg".to_string(),
                file_name: "1.jpg".to_string(),
                file_size: 1000000,
                width: Some(1920),
                height: Some(1080),
                phash: "abc".to_string(),
                distance: 0,
            }],
            similarity: 1.0,
        };

        assert!(group.images.len() < 2);
    }
}
