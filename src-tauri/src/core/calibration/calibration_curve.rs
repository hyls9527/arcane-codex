use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSegment {
    pub raw_lower: f64,
    pub raw_upper: f64,
    pub calibrated_value: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCalibrationCurve {
    pub category: String,
    pub segments: Vec<CalibrationSegment>,
    pub total_samples: usize,
    pub raw_ece: f64,
    pub calibrated_ece: f64,
}

pub struct CalibrationMapper {
    curves: Vec<CategoryCalibrationCurve>,
}

impl CalibrationMapper {
    pub fn from_curves(curves: Vec<CategoryCalibrationCurve>) -> Self {
        Self { curves }
    }

    pub fn calibrate(&self, category: &str, raw_confidence: f64) -> f64 {
        let curve = self.curves.iter().find(|c| c.category == category);
        let curve = match curve {
            Some(c) => c,
            None => return raw_confidence,
        };

        for segment in &curve.segments {
            if raw_confidence >= segment.raw_lower && raw_confidence < segment.raw_upper {
                return segment.calibrated_value;
            }
        }

        if let Some(last) = curve.segments.last() {
            if raw_confidence >= last.raw_upper - f64::EPSILON {
                return last.calibrated_value;
            }
        }

        raw_confidence
    }

    pub fn calibrate_batch(
        &self,
        results: &[(String, f64)],
    ) -> Vec<f64> {
        results
            .iter()
            .map(|(category, confidence)| self.calibrate(category, *confidence))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibrate_within_segment() {
        let curve = CategoryCalibrationCurve {
            category: "动物".to_string(),
            segments: vec![
                CalibrationSegment {
                    raw_lower: 0.0,
                    raw_upper: 0.5,
                    calibrated_value: 0.3,
                    sample_count: 10,
                },
                CalibrationSegment {
                    raw_lower: 0.5,
                    raw_upper: 1.0,
                    calibrated_value: 0.8,
                    sample_count: 20,
                },
            ],
            total_samples: 30,
            raw_ece: 0.2,
            calibrated_ece: 0.05,
        };

        let mapper = CalibrationMapper::from_curves(vec![curve]);

        let calibrated = mapper.calibrate("动物", 0.7);
        assert!((calibrated - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calibrate_unknown_category_returns_raw() {
        let curve = CategoryCalibrationCurve {
            category: "动物".to_string(),
            segments: vec![],
            total_samples: 0,
            raw_ece: 0.0,
            calibrated_ece: 0.0,
        };

        let mapper = CalibrationMapper::from_curves(vec![curve]);
        let calibrated = mapper.calibrate("未知类别", 0.75);
        assert!((calibrated - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calibrate_boundary_values() {
        let curve = CategoryCalibrationCurve {
            category: "动物".to_string(),
            segments: vec![
                CalibrationSegment {
                    raw_lower: 0.0,
                    raw_upper: 0.5,
                    calibrated_value: 0.3,
                    sample_count: 10,
                },
            ],
            total_samples: 10,
            raw_ece: 0.2,
            calibrated_ece: 0.05,
        };

        let mapper = CalibrationMapper::from_curves(vec![curve]);

        let low = mapper.calibrate("动物", 0.25);
        assert!((low - 0.3).abs() < f64::EPSILON);

        let high = mapper.calibrate("动物", 0.9999);
        assert!((high - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calibrate_batch() {
        let curve1 = CategoryCalibrationCurve {
            category: "动物".to_string(),
            segments: vec![
                CalibrationSegment {
                    raw_lower: 0.0,
                    raw_upper: 1.0,
                    calibrated_value: 0.7,
                    sample_count: 10,
                },
            ],
            total_samples: 10,
            raw_ece: 0.2,
            calibrated_ece: 0.05,
        };

        let curve2 = CategoryCalibrationCurve {
            category: "文档".to_string(),
            segments: vec![
                CalibrationSegment {
                    raw_lower: 0.0,
                    raw_upper: 1.0,
                    calibrated_value: 0.9,
                    sample_count: 15,
                },
            ],
            total_samples: 15,
            raw_ece: 0.1,
            calibrated_ece: 0.02,
        };

        let mapper = CalibrationMapper::from_curves(vec![curve1, curve2]);

        let input = vec![
            ("动物".to_string(), 0.8),
            ("文档".to_string(), 0.6),
            ("动物".to_string(), 0.5),
        ];

        let calibrated = mapper.calibrate_batch(&input);
        assert_eq!(calibrated.len(), 3);
        assert!((calibrated[0] - 0.7).abs() < f64::EPSILON);
        assert!((calibrated[1] - 0.9).abs() < f64::EPSILON);
        assert!((calibrated[2] - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiple_curves_select_correct() {
        let curves = vec![
            CategoryCalibrationCurve {
                category: "动物".to_string(),
                segments: vec![
                    CalibrationSegment {
                        raw_lower: 0.0,
                        raw_upper: 1.0,
                        calibrated_value: 0.6,
                        sample_count: 10,
                    },
                ],
                total_samples: 10,
                raw_ece: 0.3,
                calibrated_ece: 0.1,
            },
            CategoryCalibrationCurve {
                category: "文档".to_string(),
                segments: vec![
                    CalibrationSegment {
                        raw_lower: 0.0,
                        raw_upper: 1.0,
                        calibrated_value: 0.95,
                        sample_count: 20,
                    },
                ],
                total_samples: 20,
                raw_ece: 0.05,
                calibrated_ece: 0.01,
            },
        ];

        let mapper = CalibrationMapper::from_curves(curves);

        let animal_result = mapper.calibrate("动物", 0.8);
        let document_result = mapper.calibrate("文档", 0.8);

        assert!((animal_result - 0.6).abs() < f64::EPSILON);
        assert!((document_result - 0.95).abs() < f64::EPSILON);
        assert!(animal_result != document_result, "Different categories should have different calibrated values");
    }
}
