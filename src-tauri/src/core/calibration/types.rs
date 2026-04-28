use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageCategory {
    Landscape,
    Person,
    Object,
    Animal,
    Architecture,
    Document,
    Other,
}

impl ImageCategory {
    pub fn from_str(s: &str) -> Self {
        match s {
            "风景" | "landscape" => Self::Landscape,
            "人物" | "person" => Self::Person,
            "物品" | "object" => Self::Object,
            "动物" | "animal" => Self::Animal,
            "建筑" | "architecture" | "building" => Self::Architecture,
            "文档" | "document" => Self::Document,
            _ => Self::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Landscape => "风景",
            Self::Person => "人物",
            Self::Object => "物品",
            Self::Animal => "动物",
            Self::Architecture => "建筑",
            Self::Document => "文档",
            Self::Other => "其他",
        }
    }

    pub fn all() -> &'static [ImageCategory] {
        &[
            Self::Landscape,
            Self::Person,
            Self::Object,
            Self::Animal,
            Self::Architecture,
            Self::Document,
            Self::Other,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct CalibrationSample {
    pub image_id: i64,
    pub predicted_category: ImageCategory,
    pub raw_confidence: f64,
    pub is_correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBin {
    pub bin_index: usize,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
    pub sample_count: usize,
    pub avg_confidence: f64,
    pub avg_accuracy: f64,
    pub gap: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub overall_ece: f64,
    pub per_category_ece: HashMap<String, f64>,
    pub total_samples: usize,
    pub bins: Vec<CalibrationBin>,
    pub computed_at: String,
}

#[derive(Debug, Clone)]
pub struct CalibrationConfig {
    pub num_bins: usize,
    pub min_samples_per_bin: usize,
    pub enable_per_category: bool,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            num_bins: 10,
            min_samples_per_bin: 5,
            enable_per_category: true,
        }
    }
}
