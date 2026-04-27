use tauri::State;
use serde::{Deserialize, Serialize};
use crate::core::db::Database;
use crate::core::search_index::{SearchIndexBuilder, SearchFilters, SearchResult};
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub page: u32,
    pub page_size: u32,
}

#[tauri::command]
pub async fn semantic_search(
    db: State<'_, Database>,
    request: SearchRequest,
) -> AppResult<SearchResponse> {
    if request.query.trim().is_empty() {
        return Err(AppError::Validation("搜索查询不能为空".to_string()));
    }

    let builder = SearchIndexBuilder::new();

    let filters = SearchFilters {
        category: request.category,
        tags: request.tags,
        start_date: request.start_date,
        end_date: request.end_date,
    };

    let page = request.page.unwrap_or(0);
    let page_size = request.page_size.unwrap_or(20);
    let offset = (page * page_size) as usize;

    let results = builder.search(
        &db,
        &request.query,
        Some(&filters),
        page_size as usize,
        offset,
    )?;

    let total = results.len();

    Ok(SearchResponse {
        results,
        total,
        page,
        page_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_search_cmd.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_search_empty_query() {
        let request = SearchRequest {
            query: "".to_string(),
            category: None,
            tags: None,
            start_date: None,
            end_date: None,
            page: None,
            page_size: None,
        };

        assert!(request.query.trim().is_empty());
    }

    #[test]
    fn test_search_request_defaults() {
        let request = SearchRequest {
            query: "cat".to_string(),
            category: None,
            tags: None,
            start_date: None,
            end_date: None,
            page: None,
            page_size: None,
        };

        assert_eq!(request.page.unwrap_or(0), 0);
        assert_eq!(request.page_size.unwrap_or(20), 20);
    }

    #[test]
    fn test_search_response_structure() {
        let response = SearchResponse {
            results: vec![],
            total: 0,
            page: 0,
            page_size: 20,
        };

        assert!(response.results.is_empty());
        assert_eq!(response.total, 0);
        assert_eq!(response.page, 0);
        assert_eq!(response.page_size, 20);
    }

    #[test]
    fn test_search_with_filters() {
        let request = SearchRequest {
            query: "mountain landscape".to_string(),
            category: Some("landscape".to_string()),
            tags: Some(vec!["nature".to_string(), "outdoor".to_string()]),
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            page: Some(1),
            page_size: Some(10),
        };

        assert_eq!(request.query, "mountain landscape");
        assert_eq!(request.category, Some("landscape".to_string()));
        assert_eq!(request.tags.as_ref().unwrap().len(), 2);
        assert_eq!(request.page, Some(1));
        assert_eq!(request.page_size, Some(10));
    }

    #[test]
    fn test_search_pagination_calculation() {
        let page = 2u32;
        let page_size = 15u32;
        let offset = (page * page_size) as usize;

        assert_eq!(offset, 30);
    }
}
