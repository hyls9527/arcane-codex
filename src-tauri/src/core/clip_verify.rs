use serde::{Deserialize, Serialize};
use crate::core::calibration::types::ImageCategory;
use crate::utils::error::{AppResult, AppError};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipClassificationResult {
    pub category: ImageCategory,
    pub confidence: f64,
    pub top_categories: Vec<(ImageCategory, f64)>,
}

#[async_trait]
pub trait ClipVerifier: Send + Sync {
    fn name(&self) -> &str;
    async fn classify_image(&self, image_path: &str) -> AppResult<ClipClassificationResult>;
    async fn health_check(&self) -> bool;
}

pub fn categories_to_clip_labels() -> Vec<&'static str> {
    vec![
        "landscape photo, nature, scenery",
        "portrait photo, person, people",
        "object photo, product, still life",
        "animal photo, pet, wildlife",
        "architecture photo, building, structure",
        "document, screenshot, text image",
        "other, abstract, miscellaneous",
    ]
}

pub fn clip_result_to_category(clip_category: &str) -> ImageCategory {
    match clip_category {
        "landscape" => ImageCategory::Landscape,
        "person" => ImageCategory::Person,
        "object" => ImageCategory::Object,
        "animal" => ImageCategory::Animal,
        "architecture" => ImageCategory::Architecture,
        "document" => ImageCategory::Document,
        _ => ImageCategory::Other,
    }
}

pub fn check_category_agreement(
    visual_category: &ImageCategory,
    clip_category: &ImageCategory,
) -> bool {
    visual_category == clip_category
}

pub fn check_agreement_with_threshold(
    visual_confidence: f64,
    clip_confidence: f64,
    category_agreement: bool,
    min_confidence: f64,
) -> bool {
    if !category_agreement {
        return false;
    }
    visual_confidence >= min_confidence && clip_confidence >= min_confidence
}

pub fn generate_rejection_reason(
    visual_category: &ImageCategory,
    clip_category: &ImageCategory,
    visual_confidence: f64,
    clip_confidence: f64,
) -> String {
    format!(
        "类别不一致: 视觉模型={}, CLIP={}, 视觉置信度={:.2}, CLIP 置信度={:.2}",
        visual_category.as_str(),
        clip_category.as_str(),
        visual_confidence,
        clip_confidence
    )
}

#[derive(Debug)]
pub struct MockClipVerifier {
    default_category: ImageCategory,
    default_confidence: f64,
}

impl MockClipVerifier {
    pub fn new(category: ImageCategory, confidence: f64) -> Self {
        Self {
            default_category: category,
            default_confidence: confidence,
        }
    }
}

#[async_trait]
impl ClipVerifier for MockClipVerifier {
    fn name(&self) -> &str {
        "mock_clip"
    }

    async fn classify_image(&self, _image_path: &str) -> AppResult<ClipClassificationResult> {
        let result = ClipClassificationResult {
            category: self.default_category.clone(),
            confidence: self.default_confidence,
            top_categories: vec![
                (self.default_category.clone(), self.default_confidence),
                (ImageCategory::Other, 1.0 - self.default_confidence),
            ],
        };
        Ok(result)
    }

    async fn health_check(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categories_to_clip_labels_length() {
        let labels = categories_to_clip_labels();
        assert_eq!(labels.len(), 7, "Should have 7 CLIP labels matching ImageCategory variants");
    }

    #[test]
    fn test_clip_result_to_category_mapping() {
        assert_eq!(clip_result_to_category("landscape"), ImageCategory::Landscape);
        assert_eq!(clip_result_to_category("person"), ImageCategory::Person);
        assert_eq!(clip_result_to_category("object"), ImageCategory::Object);
        assert_eq!(clip_result_to_category("animal"), ImageCategory::Animal);
        assert_eq!(clip_result_to_category("architecture"), ImageCategory::Architecture);
        assert_eq!(clip_result_to_category("document"), ImageCategory::Document);
        assert_eq!(clip_result_to_category("unknown"), ImageCategory::Other);
    }

    #[test]
    fn test_check_category_agreement_same() {
        let visual = ImageCategory::Animal;
        let clip = ImageCategory::Animal;
        assert!(check_category_agreement(&visual, &clip));
    }

    #[test]
    fn test_check_category_agreement_different() {
        let visual = ImageCategory::Animal;
        let clip = ImageCategory::Document;
        assert!(!check_category_agreement(&visual, &clip));
    }

    #[test]
    fn test_check_agreement_with_threshold_pass() {
        let result = check_agreement_with_threshold(0.9, 0.85, true, 0.7);
        assert!(result);
    }

    #[test]
    fn test_check_agreement_with_threshold_category_mismatch() {
        let result = check_agreement_with_threshold(0.9, 0.85, false, 0.7);
        assert!(!result, "Should reject even with high confidence when categories differ");
    }

    #[test]
    fn test_check_agreement_with_threshold_low_visual_confidence() {
        let result = check_agreement_with_threshold(0.5, 0.85, true, 0.7);
        assert!(!result, "Should reject when visual confidence is below threshold");
    }

    #[test]
    fn test_check_agreement_with_threshold_low_clip_confidence() {
        let result = check_agreement_with_threshold(0.9, 0.5, true, 0.7);
        assert!(!result, "Should reject when CLIP confidence is below threshold");
    }

    #[test]
    fn test_generate_rejection_reason() {
        let reason = generate_rejection_reason(
            &ImageCategory::Animal,
            &ImageCategory::Document,
            0.85,
            0.30,
        );
        assert!(reason.contains("动物"));
        assert!(reason.contains("文档"));
        assert!(reason.contains("0.85"));
        assert!(reason.contains("0.30"));
    }

    #[test]
    fn test_mock_clip_verifier_classify() {
        let verifier = MockClipVerifier::new(ImageCategory::Landscape, 0.92);
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(verifier.classify_image("test.jpg")).unwrap();
        
        assert_eq!(result.category, ImageCategory::Landscape);
        assert!((result.confidence - 0.92).abs() < f64::EPSILON);
        assert_eq!(result.top_categories.len(), 2);
    }

    #[test]
    fn test_mock_clip_verifier_health_check() {
        let verifier = MockClipVerifier::new(ImageCategory::Other, 0.5);
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let healthy = rt.block_on(verifier.health_check());
        
        assert!(healthy, "Mock verifier should always report healthy");
    }

    #[test]
    fn test_mock_clip_verifier_name() {
        let verifier = MockClipVerifier::new(ImageCategory::Animal, 0.8);
        assert_eq!(verifier.name(), "mock_clip");
    }
}
