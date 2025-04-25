pub mod grade;
pub mod scale;

use scale::{GradeScaleType, GradingScale};

#[derive(Debug, Default)]
pub struct Model {
    pub scale: GradingScale,
    pub students: Option<Vec<(String, f64)>>,
}

impl Model {
    pub fn new() -> Self {
        let scale = GradingScale::from_type(GradeScaleType::IHK, 100.0).unwrap(); // todo! better error handling.

        Self {
            scale,
            students: None,
        }
    }
}
