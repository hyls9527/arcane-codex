pub mod ece;
pub mod calibration_curve;
pub mod types;

pub use ece::EceCalculator;
pub use calibration_curve::CalibrationMapper;
pub use types::{CalibrationConfig, CalibrationSample, CalibrationReport};

use crate::core::calibration::types::ImageCategory;
use crate::core::calibration::calibration_curve::{CalibrationSegment, CategoryCalibrationCurve};

pub struct CalibrationService {
    calculator: EceCalculator,
    curves: Vec<CategoryCalibrationCurve>,
}

impl CalibrationService {
    pub fn new(config: CalibrationConfig) -> Self {
        Self {
            calculator: EceCalculator::new(config),
            curves: Vec::new(),
        }
    }

    pub fn generate_report(&self, samples: &[CalibrationSample]) -> CalibrationReport {
        self.calculator.generate_report(samples)
    }

    pub fn compute_ece(&self, samples: &[CalibrationSample]) -> f64 {
        self.calculator.compute_ece(samples)
    }

    pub fn compute_per_category_ece(
        &self,
        samples: &[CalibrationSample],
    ) -> std::collections::HashMap<ImageCategory, f64> {
        self.calculator.compute_per_category_ece(samples)
    }

    pub fn rebuild_curves(&mut self, samples: &[CalibrationSample]) {
        let num_bins = self.calculator.config().num_bins;
        let bin_width = 1.0 / num_bins as f64;

        self.curves.clear();

        for category in ImageCategory::all() {
            let cat_samples: Vec<CalibrationSample> = samples
                .iter()
                .filter(|s| s.predicted_category == *category)
                .cloned()
                .collect();

            if cat_samples.is_empty() {
                continue;
            }

            let raw_ece = self.calculator.compute_ece(&cat_samples);

            let segments: Vec<CalibrationSegment> = (0..num_bins)
                .map(|i| {
                    let lower = i as f64 * bin_width;
                    let upper = (i + 1) as f64 * bin_width;

                    let bucket: Vec<&CalibrationSample> = cat_samples
                        .iter()
                        .filter(|s| s.raw_confidence >= lower && s.raw_confidence < upper)
                        .collect();

                    let count = bucket.len();
                    let avg_acc = if count == 0 {
                        lower + bin_width / 2.0
                    } else {
                        bucket.iter().map(|s| s.is_correct as u8 as f64).sum::<f64>() / count as f64
                    };

                    CalibrationSegment {
                        raw_lower: lower,
                        raw_upper: upper,
                        calibrated_value: avg_acc,
                        sample_count: count,
                    }
                })
                .collect();

            let curve = CategoryCalibrationCurve {
                category: category.as_str().to_string(),
                segments,
                total_samples: cat_samples.len(),
                raw_ece,
                calibrated_ece: 0.0,
            };

            self.curves.push(curve);
        }
    }

    pub fn get_mapper(&self) -> CalibrationMapper {
        CalibrationMapper::from_curves(self.curves.clone())
    }

    pub fn calibrate(&self, category: &str, raw_confidence: f64) -> f64 {
        let mapper = self.get_mapper();
        mapper.calibrate(category, raw_confidence)
    }

    pub fn curves(&self) -> &[CategoryCalibrationCurve] {
        &self.curves
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(image_id: i64, category: ImageCategory, confidence: f64, correct: bool) -> CalibrationSample {
        CalibrationSample {
            image_id,
            predicted_category: category,
            raw_confidence: confidence,
            is_correct: correct,
        }
    }

    #[test]
    fn test_service_generate_report() {
        let service = CalibrationService::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.9, true),
            sample(2, ImageCategory::Animal, 0.8, true),
            sample(3, ImageCategory::Document, 0.7, false),
        ];

        let report = service.generate_report(&samples);

        assert_eq!(report.total_samples, 3);
        assert!(report.overall_ece >= 0.0);
        assert!(!report.per_category_ece.is_empty());
    }

    #[test]
    fn test_service_rebuild_curves() {
        let mut service = CalibrationService::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.9, true),
            sample(2, ImageCategory::Animal, 0.8, true),
            sample(3, ImageCategory::Animal, 0.7, true),
            sample(4, ImageCategory::Document, 0.6, false),
            sample(5, ImageCategory::Document, 0.5, false),
        ];

        service.rebuild_curves(&samples);

        assert!(!service.curves.is_empty());
        assert!(service.curves.len() <= 2, "Should have Animal and Document curves only");
        
        let animal_curve = service.curves.iter().find(|c| c.category == "动物");
        assert!(animal_curve.is_some());
    }

    #[test]
    fn test_service_calibrate() {
        let mut service = CalibrationService::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.9, true),
            sample(2, ImageCategory::Animal, 0.9, true),
            sample(3, ImageCategory::Animal, 0.8, true),
        ];

        service.rebuild_curves(&samples);

        let calibrated = service.calibrate("动物", 0.85);
        assert!(calibrated >= 0.0 && calibrated <= 1.0, "Calibrated value should be in [0, 1]");
    }

    #[test]
    fn test_service_calibrate_unknown_category() {
        let mut service = CalibrationService::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.9, true),
        ];

        service.rebuild_curves(&samples);

        let calibrated = service.calibrate("不存在的类别", 0.75);
        assert!((calibrated - 0.75).abs() < f64::EPSILON, "Unknown category should return raw confidence");
    }

    #[test]
    fn test_service_empty_samples() {
        let mut service = CalibrationService::new(CalibrationConfig::default());
        
        let samples: Vec<CalibrationSample> = vec![];

        let report = service.generate_report(&samples);
        assert_eq!(report.total_samples, 0);
        assert_eq!(report.overall_ece, 0.0);

        service.rebuild_curves(&samples);
        assert!(service.curves.is_empty());
    }
}
