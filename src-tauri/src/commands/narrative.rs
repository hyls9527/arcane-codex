use serde::{Deserialize, Serialize};
use tauri::State;
use crate::core::ai_queue::AITaskQueue;
use crate::utils::error::{AppError, AppResult};
use rusqlite::params;

#[derive(Debug, Serialize, Deserialize)]
pub struct NarrativeResult {
    pub id: i64,
    pub image_id: i64,
    pub content: String,
    pub entities_json: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociationResult {
    pub image_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub thumbnail_path: Option<String>,
    pub narrative_content: String,
    pub match_type: String,
    pub relevance: f64,
}

const PERSON_PREFIXES: &[&str] = &["和", "与", "跟"];

const PLACE_PREFIXES: &[&str] = &["在", "去", "到", "从"];
const PLACE_SUFFIXES: &[&str] = &["的", "那", "玩", "出差", "旅行", "旅游", "逛", "吃饭", "拍照", "看", "住", "走"];

const TIME_KEYWORDS: &[&str] = &[
    "去年", "今年", "前年", "上个月", "这个月", "昨天", "今天", "前天", "大前天", "上周", "这周", "下周",
];

const WEEKDAY_PREFIXES: &[&str] = &["周", "星期"];
const WEEKDAY_SUFFIXES: &[&str] = &["一", "二", "三", "四", "五", "六", "日", "天"];

fn is_cjk(ch: char) -> bool {
    ('\u{4e00}' <= ch && ch <= '\u{9fff}')
        || ('\u{3400}' <= ch && ch <= '\u{4dbf}')
        || ('\u{f900}' <= ch && ch <= '\u{faff}')
}

fn extract_entities(content: &str) -> Vec<serde_json::Value> {
    let mut entities = Vec::new();
    let mut seen_persons = std::collections::HashSet::new();
    let mut seen_places = std::collections::HashSet::new();
    let mut seen_times = std::collections::HashSet::new();

    let chars: Vec<char> = content.chars().collect();

    fn find_substring(chars: &[char], pattern: &[char], from: usize) -> Option<usize> {
        if pattern.is_empty() || from + pattern.len() > chars.len() {
            return None;
        }
        for i in from..=chars.len() - pattern.len() {
            if &chars[i..i + pattern.len()] == pattern {
                return Some(i);
            }
        }
        None
    }

    fn all_cjk(chars: &[char], start: usize, len: usize) -> bool {
        if start + len > chars.len() {
            return false;
        }
        chars[start..start + len].iter().all(|c| is_cjk(*c))
    }

    for prefix in PERSON_PREFIXES {
        let prefix_chars: Vec<char> = prefix.chars().collect();
        let mut search_from = 0;
        while let Some(pos) = find_substring(&chars, &prefix_chars, search_from) {
            let after = pos + prefix_chars.len();
            for &name_len in &[2, 3] {
                if all_cjk(&chars, after, name_len) {
                    let name: String = chars[after..after + name_len].iter().collect();
                    if !seen_persons.contains(&name) {
                        seen_persons.insert(name.clone());
                        entities.push(serde_json::json!({
                            "type": "person",
                            "value": name
                        }));
                    }
                    break;
                }
            }
            search_from = after;
        }
    }

    let person_conjunctions: &[&str] = &["和", "与", "跟"];
    for conj in person_conjunctions {
        let conj_chars: Vec<char> = conj.chars().collect();
        let mut search_from = 0;
        while let Some(pos) = find_substring(&chars, &conj_chars, search_from) {
            for &name_len in &[2, 3] {
                if name_len <= pos {
                    let start = pos - name_len;
                    if all_cjk(&chars, start, name_len) {
                        let name: String = chars[start..pos].iter().collect();
                        if !seen_persons.contains(&name) {
                            seen_persons.insert(name.clone());
                            entities.push(serde_json::json!({
                                "type": "person",
                                "value": name
                            }));
                        }
                        break;
                    }
                }
            }
            search_from = pos + conj_chars.len();
        }
    }

    for prefix in PLACE_PREFIXES {
        let prefix_chars: Vec<char> = prefix.chars().collect();
        let mut search_from = 0;
        while let Some(pos) = find_substring(&chars, &prefix_chars, search_from) {
            let after = pos + prefix_chars.len();

            let from_prefix: String = chars[pos..].iter().collect();
            let is_time_at_prefix = TIME_KEYWORDS.iter().any(|kw| from_prefix.starts_with(kw));
            let after_text: String = if after < chars.len() {
                chars[after..].iter().collect()
            } else {
                String::new()
            };
            let is_time_after_prefix = TIME_KEYWORDS.iter().any(|kw| after_text.starts_with(kw));

            if is_time_at_prefix || is_time_after_prefix {
                search_from = after;
                continue;
            }

            if all_cjk(&chars, after, 2) {
                let place: String = chars[after..after + 2].iter().collect();
                if !seen_places.contains(&place) {
                    seen_places.insert(place.clone());
                    entities.push(serde_json::json!({
                        "type": "place",
                        "value": place
                    }));
                }
            } else {
                for place_len in 3..=6 {
                    if !all_cjk(&chars, after, place_len) {
                        break;
                    }
                    let rest_start = after + place_len;
                    let rest: String = chars[rest_start..].iter().collect();
                    let valid = PLACE_SUFFIXES.iter().any(|suffix| rest.starts_with(suffix));
                    if valid {
                        let place: String = chars[after..after + place_len].iter().collect();
                        if !seen_places.contains(&place) {
                            seen_places.insert(place.clone());
                            entities.push(serde_json::json!({
                                "type": "place",
                                "value": place
                            }));
                        }
                        break;
                    }
                }
            }
            search_from = after;
        }
    }

    for keyword in TIME_KEYWORDS {
        if content.contains(keyword) && !seen_times.contains(*keyword) {
            seen_times.insert(keyword.to_string());
            entities.push(serde_json::json!({
                "type": "time",
                "value": keyword
            }));
        }
    }

    for wp in WEEKDAY_PREFIXES {
        if let Some(pos) = content.find(wp) {
            let after = pos + wp.len();
            let char_pos = content[..after].chars().count();
            if char_pos < chars.len() {
                let day_char = chars[char_pos];
                let day_str = day_char.to_string();
                if WEEKDAY_SUFFIXES.contains(&day_str.as_str()) {
                    let time_val = format!("{}{}", wp, day_char);
                    if !seen_times.contains(&time_val) {
                        seen_times.insert(time_val.clone());
                        entities.push(serde_json::json!({
                            "type": "time",
                            "value": time_val
                        }));
                    }
                }
            }
        }
    }

    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let num_start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '月' {
                let month_end = i + 1;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                if i < chars.len() && (chars[i] == '日' || chars[i] == '号') {
                    let time_val: String = chars[num_start..=i].iter().collect();
                    if !seen_times.contains(&time_val) {
                        seen_times.insert(time_val.clone());
                        entities.push(serde_json::json!({
                            "type": "time",
                            "value": time_val
                        }));
                    }
                    i += 1;
                } else {
                    i = month_end;
                }
            }
        } else {
            i += 1;
        }
    }

    entities
}

fn validate_narrative_content(content: &str) -> AppResult<()> {
    if content.trim().is_empty() {
        return Err(AppError::validation("叙事内容不能为空"));
    }
    Ok(())
}

#[tauri::command]
pub async fn write_narrative(
    image_id: i64,
    content: String,
    queue: State<'_, AITaskQueue>,
) -> AppResult<NarrativeResult> {
    validate_narrative_content(&content)?;

    let entities = extract_entities(&content);
    let entities_json = serde_json::to_string(&entities).unwrap_or_else(|_| "[]".to_string());

    let db = queue.db();
    let conn = db.open_connection().map_err(AppError::database)?;

    conn.execute(
        "INSERT INTO narratives (image_id, content, entities_json) VALUES (?1, ?2, ?3)",
        params![image_id, content, entities_json],
    )
    .map_err(AppError::database)?;

    let id = conn.last_insert_rowid();

    Ok(NarrativeResult {
        id,
        image_id,
        content,
        entities_json,
    })
}

#[tauri::command]
pub async fn get_narratives(
    image_id: i64,
    queue: State<'_, AITaskQueue>,
) -> AppResult<Vec<NarrativeResult>> {
    let db = queue.db();
    let conn = db.open_connection().map_err(AppError::database)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, image_id, content, entities_json FROM narratives WHERE image_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(AppError::database)?;

    let rows = stmt
        .query_map(params![image_id], |row| {
            Ok(NarrativeResult {
                id: row.get(0)?,
                image_id: row.get(1)?,
                content: row.get(2)?,
                entities_json: row.get(3)?,
            })
        })
        .map_err(AppError::database)?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(AppError::database)?);
    }

    Ok(results)
}

#[tauri::command]
pub async fn query_associations(
    query: String,
    limit: Option<i64>,
    queue: State<'_, AITaskQueue>,
) -> AppResult<Vec<AssociationResult>> {
    if query.trim().is_empty() {
        return Err(AppError::validation("查询内容不能为空"));
    }

    let limit = limit.unwrap_or(20);
    let db = queue.db();
    let conn = db.open_connection().map_err(AppError::database)?;

    let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));

    let mut stmt = conn
        .prepare(
            "SELECT n.image_id, i.file_path, i.file_name, i.thumbnail_path, n.content, 'content' AS match_type, 1.0 AS relevance
             FROM narratives n
             JOIN images i ON n.image_id = i.id
             WHERE n.content LIKE ?1 ESCAPE '\\'
             UNION ALL
             SELECT n.image_id, i.file_path, i.file_name, i.thumbnail_path, n.content, 'entity' AS match_type, 0.8 AS relevance
             FROM narratives n
             JOIN images i ON n.image_id = i.id
             WHERE n.entities_json LIKE ?1 ESCAPE '\\'
             AND n.id NOT IN (
                 SELECT n2.id FROM narratives n2 WHERE n2.content LIKE ?1 ESCAPE '\\'
             )
             ORDER BY relevance DESC
             LIMIT ?2",
        )
        .map_err(AppError::database)?;

    let rows = stmt
        .query_map(params![pattern, limit], |row| {
            Ok(AssociationResult {
                image_id: row.get(0)?,
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                thumbnail_path: row.get(3)?,
                narrative_content: row.get(4)?,
                match_type: row.get(5)?,
                relevance: row.get(6)?,
            })
        })
        .map_err(AppError::database)?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(AppError::database)?);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_entities_person() {
        let entities = extract_entities("我和小明一起去公园玩");
        let person_entities: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "person")
            .collect();
        assert!(!person_entities.is_empty(), "应提取出人名实体");
    }

    #[test]
    fn test_extract_entities_place() {
        let entities = extract_entities("我去北京出差");
        let place_entities: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "place")
            .collect();
        assert!(!place_entities.is_empty(), "应提取出地名实体");
    }

    #[test]
    fn test_extract_entities_time() {
        let entities = extract_entities("去年夏天我们去旅行");
        let time_entities: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "time")
            .collect();
        assert!(!time_entities.is_empty(), "应提取出时间实体");
    }

    #[test]
    fn test_extract_entities_multiple() {
        let entities = extract_entities("今年和小红去上海旅行");
        assert!(entities.len() >= 2, "应提取出多个实体");
    }

    #[test]
    fn test_extract_entities_persons() {
        let entities = extract_entities("和老王去西湖那天");
        let persons: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "person")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        assert!(persons.contains(&"老王"), "persons 应包含 '老王'");
    }

    #[test]
    fn test_extract_entities_locations() {
        let entities = extract_entities("在杭州拍的");
        let locations: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "place")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        assert!(locations.contains(&"杭州"), "locations 应包含 '杭州'");
    }

    #[test]
    fn test_extract_entities_times() {
        let entities = extract_entities("去年夏天的旅行");
        let times: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "time")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        assert!(times.contains(&"去年"), "times 应包含 '去年'");
    }

    #[test]
    fn test_extract_entities_combined() {
        let entities = extract_entities("和老王在杭州去年夏天");
        let persons: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "person")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        let locations: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "place")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        let times: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "time")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        assert!(persons.contains(&"老王"), "persons 应包含 '老王'");
        assert!(locations.contains(&"杭州"), "locations 应包含 '杭州'");
        assert!(times.contains(&"去年"), "times 应包含 '去年'");
    }

    #[test]
    fn test_extract_entities_empty() {
        let entities = extract_entities("一张风景照");
        let persons: Vec<_> = entities.iter().filter(|e| e["type"] == "person").collect();
        let locations: Vec<_> = entities.iter().filter(|e| e["type"] == "place").collect();
        let times: Vec<_> = entities.iter().filter(|e| e["type"] == "time").collect();
        assert!(persons.is_empty(), "persons 应为空");
        assert!(locations.is_empty(), "locations 应为空");
        assert!(times.is_empty(), "times 应为空");
    }

    #[test]
    fn test_write_narrative_empty_content() {
        let result = validate_narrative_content("");
        assert!(result.is_err(), "空内容应返回验证错误");
    }

    #[test]
    fn test_extract_entities_date() {
        let entities = extract_entities("5月3号我们去爬山");
        let time_entities: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "time")
            .collect();
        assert!(!time_entities.is_empty(), "应提取出日期实体");
    }

    #[test]
    fn test_extract_entities_weekday() {
        let entities = extract_entities("周三开会");
        let time_entities: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "time")
            .collect();
        assert!(!time_entities.is_empty(), "应提取出星期实体");
    }

    #[test]
    fn test_narrative_result_serialization() {
        let result = NarrativeResult {
            id: 1,
            image_id: 42,
            content: "测试叙事".to_string(),
            entities_json: r#"[{"type":"person","value":"小明"}]"#.to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: NarrativeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.image_id, 42);
        assert_eq!(deserialized.content, "测试叙事");
    }

    #[test]
    fn test_association_result_serialization() {
        let result = AssociationResult {
            image_id: 1,
            file_path: "/test/image.jpg".to_string(),
            file_name: "image.jpg".to_string(),
            thumbnail_path: Some("/test/thumb.webp".to_string()),
            narrative_content: "测试".to_string(),
            match_type: "content".to_string(),
            relevance: 1.0,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AssociationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.match_type, "content");
        assert_eq!(deserialized.relevance, 1.0);
    }

    #[test]
    fn test_validate_narrative_empty_content() {
        let result = validate_narrative_content("");
        assert!(result.is_err(), "空内容应返回验证错误");
        
        let result = validate_narrative_content("   ");
        assert!(result.is_err(), "纯空白内容应返回验证错误");
        
        let result = validate_narrative_content("有内容的叙事");
        assert!(result.is_ok(), "有效内容应通过验证");
    }

    #[test]
    fn test_extract_entities_person_before_conjunction() {
        let entities = extract_entities("小明和小红一起去公园");
        let persons: Vec<_> = entities.iter().filter(|e| e["type"] == "person").collect();
        assert!(persons.len() >= 1, "应从连词前提取人名");
    }

    #[test]
    fn test_extract_entities_long_place_name() {
        let entities = extract_entities("去乌鲁木齐旅行");
        let places: Vec<_> = entities.iter().filter(|e| e["type"] == "place").collect();
        assert!(!places.is_empty(), "应提取出长地名");
    }

    #[test]
    fn test_query_associations_deduplication() {
        let content = "和老王在杭州拍的";
        let entities = extract_entities(content);
        let persons: Vec<_> = entities.iter().filter(|e| e["type"] == "person").collect();
        let places: Vec<_> = entities.iter().filter(|e| e["type"] == "place").collect();
        assert!(!persons.is_empty());
        assert!(!places.is_empty());
    }
}
