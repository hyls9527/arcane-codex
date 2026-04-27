use serde::{Deserialize, Serialize};
use tauri::State;
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct NarrativeResult {
    pub image_id: i64,
    pub content: String,
    pub entities: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociationResult {
    pub image_id: i64,
    pub associations: Vec<serde_json::Value>,
}

const PERSON_PREFIXES: &[&str] = &["和", "与", "跟"];
const PLACE_PREFIXES: &[&str] = &["在", "去", "到", "从"];
const PLACE_SUFFIXES: &[&str] = &["的", "那", "玩", "出差", "旅行", "旅游", "逛", "吃饭", "拍照", "看", "住", "走"];
const TIME_KEYWORDS: &[&str] = &[
    "今天", "昨天", "前天", "明天", "后天",
    "上周", "这周", "下周",
    "去年", "今年", "明年",
    "春天", "夏天", "秋天", "冬天",
    "上午", "下午", "晚上", "中午", "傍晚", "凌晨",
    "春节", "中秋", "国庆", "元旦", "端午",
];
const WEEKDAY_PREFIXES: &[&str] = &["周", "星期"];
const WEEKDAY_SUFFIXES: &[&str] = &["一", "二", "三", "四", "五", "六", "日", "天"];

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |
        '\u{3400}'..='\u{4DBF}' |
        '\u{20000}'..='\u{2A6DF}' |
        '\u{2A700}'..='\u{2B73F}' |
        '\u{2B740}'..='\u{2B81F}' |
        '\u{2B820}'..='\u{2CEAF}' |
        '\u{F900}'..='\u{FAFF}' |
        '\u{2F800}'..='\u{2FA1F}'
    )
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
        return Err(AppError::Validation("叙事内容不能为空".to_string()));
    }
    Ok(())
}

#[tauri::command]
pub async fn write_narrative(
    db: State<'_, Database>,
    image_id: i64,
    content: String,
) -> AppResult<NarrativeResult> {
    validate_narrative_content(&content)?;

    let entities = extract_entities(&content);

    let conn = db.open_connection().map_err(AppError::Database)?;

    conn.execute(
        "INSERT OR REPLACE INTO narratives (image_id, content, entities, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))",
        rusqlite::params![image_id, content, serde_json::to_string(&entities).unwrap()],
    )
    .map_err(AppError::Database)?;

    info!("写入叙事锚点: image_id={}, 实体数={}", image_id, entities.len());

    Ok(NarrativeResult {
        image_id,
        content,
        entities,
    })
}

#[tauri::command]
pub async fn get_narrative(
    db: State<'_, Database>,
    image_id: i64,
) -> AppResult<Option<NarrativeResult>> {
    let conn = db.open_connection().map_err(AppError::Database)?;

    let result = conn.query_row(
        "SELECT content, entities FROM narratives WHERE image_id = ?",
        [image_id],
        |row| {
            let content: String = row.get(0)?;
            let entities_str: String = row.get(1)?;
            let entities: Vec<serde_json::Value> = serde_json::from_str(&entities_str).unwrap_or_default();
            Ok(NarrativeResult {
                image_id,
                content,
                entities,
            })
        },
    );

    match result {
        Ok(narrative) => {
            info!("获取叙事锚点: image_id={}", image_id);
            Ok(Some(narrative))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

#[tauri::command]
pub async fn get_associations(
    db: State<'_, Database>,
    image_id: i64,
) -> AppResult<AssociationResult> {
    let conn = db.open_connection().map_err(AppError::Database)?;

    let result = conn.query_row(
        "SELECT entities FROM narratives WHERE image_id = ?",
        [image_id],
        |row| {
            let entities_str: String = row.get(0)?;
            let entities: Vec<serde_json::Value> = serde_json::from_str(&entities_str).unwrap_or_default();
            Ok(entities)
        },
    );

    let entities = result.unwrap_or_default();

    let mut associations = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for entity in &entities {
        if let Some(entity_type) = entity["type"].as_str() {
            if let Some(value) = entity["value"].as_str() {
                let key = format!("{}:{}", entity_type, value);
                if !seen.contains(&key) {
                    seen.insert(key);
                    associations.push(serde_json::json!({
                        "type": entity_type,
                        "value": value,
                        "source_image_id": image_id,
                    }));
                }
            }
        }
    }

    for entity in &entities {
        if let Some(value) = entity["value"].as_str() {
            let matches: Vec<serde_json::Value> = conn
                .prepare(&format!(
                    "SELECT image_id, entities FROM narratives WHERE image_id != ? AND entities LIKE ?"
                ))
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
                .ok()
                .map(|mut stmt| {
                    stmt.query_map(rusqlite::params![image_id, format!("%{}%", value)], |row| {
                        let id: i64 = row.get(0)?;
                        let _e_str: String = row.get(1)?;
                        Ok(serde_json::json!({
                            "image_id": id,
                            "matched_entity": value,
                        }))
                    })
                    .ok()
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
                    .unwrap_or_default()
                })
                .unwrap_or_default();

            associations.extend(matches);
        }
    }

    Ok(AssociationResult {
        image_id,
        associations,
    })
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
            image_id: 1,
            content: "和老王在杭州去年夏天".to_string(),
            entities: vec![
                serde_json::json!({"type": "person", "value": "老王"}),
            ],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: NarrativeResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.image_id, 1);
        assert_eq!(deserialized.content, "和老王在杭州去年夏天");
    }

    #[test]
    fn test_association_result_serialization() {
        let result = AssociationResult {
            image_id: 1,
            associations: vec![
                serde_json::json!({"type": "person", "value": "老王", "source_image_id": 1}),
            ],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AssociationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.image_id, 1);
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
}
