use crate::core::calibration::types::*;
use std::collections::HashMap;

pub struct EceCalculator {
    config: CalibrationConfig,
}

impl EceCalculator {
    pub fn new(config: CalibrationConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &CalibrationConfig {
        &self.config
    }

    pub fn compute_ece(&self, samples: &[CalibrationSample]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }

        let bins = self.build_bins(samples);
        let total_n = samples.len() as f64;

        bins.iter()
            .filter(|b| b.sample_count > 0)
            .map(|b| {
                let weight = b.sample_count as f64 / total_n;
                weight * b.gap
            })
            .sum()
    }

    pub fn compute_per_category_ece(
        &self,
        samples: &[CalibrationSample],
    ) -> HashMap<ImageCategory, f64> {
        let mut grouped: HashMap<ImageCategory, Vec<CalibrationSample>> = HashMap::new();
        for s in samples {
            grouped.entry(s.predicted_category.clone()).or_default().push(s.clone());
        }

        grouped
            .into_iter()
            .map(|(cat, cat_samples)| (cat, self.compute_ece(&cat_samples)))
            .collect()
    }

    pub fn generate_report(&self, samples: &[CalibrationSample]) -> CalibrationReport {
        let bins = self.build_bins(samples);
        let per_category = if self.config.enable_per_category {
            self.compute_per_category_ece(samples)
                .into_iter()
                .map(|(k, v)| (k.as_str().to_string(), v))
                .collect()
        } else {
            HashMap::new()
        };

        CalibrationReport {
            overall_ece: self.compute_ece(samples),
            per_category_ece: per_category,
            total_samples: samples.len(),
            bins,
            computed_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn build_bins(&self, samples: &[CalibrationSample]) -> Vec<CalibrationBin> {
        let num_bins = self.config.num_bins;
        let bin_width = 1.0 / num_bins as f64;

        (0..num_bins)
            .map(|i| {
                let lower = i as f64 * bin_width;
                let upper = (i + 1) as f64 * bin_width;

                let bucket: Vec<&CalibrationSample> = samples
                    .iter()
                    .filter(|s| s.raw_confidence >= lower && s.raw_confidence < upper)
                    .collect();

                let count = bucket.len();
                if count == 0 {
                    return CalibrationBin {
                        bin_index: i,
                        confidence_lower: lower,
                        confidence_upper: upper,
                        sample_count: 0,
                        avg_confidence: 0.0,
                        avg_accuracy: 0.0,
                        gap: 0.0,
                    };
                }

                let avg_conf: f64 = bucket.iter().map(|s| s.raw_confidence).sum::<f64>() / count as f64;
                let avg_acc: f64 = bucket.iter().map(|s| s.is_correct as u8 as f64).sum::<f64>() / count as f64;

                CalibrationBin {
                    bin_index: i,
                    confidence_lower: lower,
                    confidence_upper: upper,
                    sample_count: count,
                    avg_confidence: avg_conf,
                    avg_accuracy: avg_acc,
                    gap: (avg_acc - avg_conf).abs(),
                }
            })
            .collect()
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
    fn test_compute_ece_perfect_calibration() {
        let calculator = EceCalculator::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.9, true),
            sample(2, ImageCategory::Animal, 0.8, true),
            sample(3, ImageCategory::Animal, 0.7, true),
        ];

        let ece = calculator.compute_ece(&samples);
        assert!(ece < 0.2, "ECE should be low for well-calibrated predictions");
    }

    #[test]
    fn test_compute_ece_poor_calibration() {
        let calculator = EceCalculator::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.9, false),
            sample(2, ImageCategory::Animal, 0.9, false),
            sample(3, ImageCategory::Animal, 0.9, false),
        ];

        let ece = calculator.compute_ece(&samples);
        assert!(ece > 0.5, "ECE should be high when confidence != accuracy");
        assert!((ece - 0.9).abs() < 0.1, "ECE should be approximately 0.9 when model says 90% confident but 0% accurate");
    }

    #[test]
    fn test_compute_ece_empty_samples() {
        let calculator = EceCalculator::new(CalibrationConfig::default());
        let ece = calculator.compute_ece(&[]);
        assert_eq!(ece, 0.0, "ECE of empty samples should be 0");
    }

    #[test]
    fn test_compute_ece_mixed_confidence() {
        let calculator = EceCalculator::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Landscape, 0.9, true),
            sample(2, ImageCategory::Landscape, 0.8, true),
            sample(3, ImageCategory::Landscape, 0.3, false),
            sample(4, ImageCategory::Landscape, 0.2, false),
        ];

        let ece = calculator.compute_ece(&samples);
        assert!(ece > 0.0, "ECE should be positive");
        assert!(ece < 1.0, "ECE should be less than 1.0");
    }

    #[test]
    fn test_compute_per_category_ece() {
        let calculator = EceCalculator::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.9, true),
            sample(2, ImageCategory::Animal, 0.9, true),
            sample(3, ImageCategory::Document, 0.9, false),
            sample(4, ImageCategory::Document, 0.9, false),
        ];

        let per_category = calculator.compute_per_category_ece(&samples);
        
        assert!(per_category.contains_key(&ImageCategory::Animal));
        assert!(per_category.contains_key(&ImageCategory::Document));
        
        let animal_ece = per_category.get(&ImageCategory::Animal).unwrap();
        let document_ece = per_category.get(&ImageCategory::Document).unwrap();
        
        assert!(*animal_ece < *document_ece, "Animal (correct) should have lower ECE than Document (wrong)");
    }

    #[test]
    fn test_generate_report() {
        let calculator = EceCalculator::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.85, true),
            sample(2, ImageCategory::Animal, 0.75, true),
            sample(3, ImageCategory::Document, 0.60, false),
            sample(4, ImageCategory::Landscape, 0.95, true),
        ];

        let report = calculator.generate_report(&samples);

        assert_eq!(report.total_samples, 4);
        assert!(report.overall_ece >= 0.0);
        assert!(report.overall_ece <= 1.0);
        assert!(!report.bins.is_empty());
        assert!(!report.computed_at.is_empty());
    }

    #[test]
    fn test_build_bins_correct_boundaries() {
        let config = CalibrationConfig {
            num_bins: 5,
            min_samples_per_bin: 1,
            enable_per_category: false,
        };
        let calculator = EceCalculator::new(config);
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.15, true),
            sample(2, ImageCategory::Animal, 0.45, true),
            sample(3, ImageCategory::Animal, 0.85, true),
        ];

        let bins = calculator.build_bins(&samples);
        assert_eq!(bins.len(), 5);
        
        let bin_0 = &bins[0];
        assert_eq!(bin_0.bin_index, 0);
        assert!((bin_0.confidence_lower - 0.0).abs() < f64::EPSILON);
        assert!((bin_0.confidence_upper - 0.2).abs() < f64::EPSILON);
        assert_eq!(bin_0.sample_count, 1);
    }

    #[test]
    fn test_ece_weighted_calculation() {
        let calculator = EceCalculator::new(CalibrationConfig::default());
        
        let samples = vec![
            sample(1, ImageCategory::Animal, 0.95, true),
            sample(2, ImageCategory::Animal, 0.95, true),
            sample(3, ImageCategory::Animal, 0.95, true),
            sample(4, ImageCategory::Animal, 0.95, true),
            sample(5, ImageCategory::Animal, 0.95, true),
            sample(6, ImageCategory::Animal, 0.95, true),
            sample(7, ImageCategory::Animal, 0.95, true),
            sample(8, ImageCategory::Animal, 0.95, true),
            sample(9, ImageCategory::Animal, 0.95, true),
            sample(10, ImageCategory::Animal, 0.95, true),
        ];

        let ece = calculator.compute_ece(&samples);
        assert!((ece - 0.05).abs() < 0.01, "ECE should be approximately 0.05 (1.0 - 0.95)");
    }
}
