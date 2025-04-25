use std::collections::BTreeMap;

use ratatui::style::Color;
use serde::Deserialize;
use strum_macros::{EnumIter, EnumString};

use crate::ui::grading_scale_table::GradingScaleTableRowData;
use tracing::{debug, info};

const IHK_BOUNDARIES: [(u8, f64); 6] = [
    (1, 0.92),
    (2, 0.81),
    (3, 0.67),
    (4, 0.5),
    (5, 0.3),
    (6, 0.0),
];

const TECHNIKER_BOUNDARIES: [(u8, f64); 6] =
    [(1, 0.85), (2, 0.7), (3, 0.55), (4, 0.4), (5, 0.2), (6, 0.0)];

const LINEAR_BOUNDARIES: [(u8, f64); 6] = [
    (1, 0.87),
    (2, 0.6),
    (3, 0.47),
    (4, 0.3),
    (5, 0.17),
    (6, 0.0),
];

#[derive(Debug, Default, Clone, Deserialize, EnumIter)]
pub enum GradeScaleType {
    #[default]
    IHK,
    TECHNIKER,
    LINEAR,
    Custom([(u8, f64); 6]),
}

impl GradeScaleType {
    // return the boundary values for a scale.
    pub fn values(&self) -> [(u8, f64); 6] {
        match self {
            GradeScaleType::IHK => IHK_BOUNDARIES,
            GradeScaleType::TECHNIKER => TECHNIKER_BOUNDARIES,
            GradeScaleType::LINEAR => LINEAR_BOUNDARIES,
            GradeScaleType::Custom(values) => *values,
        }
    }

    // return a text representation of the scale
    pub fn text(&self) -> &'static str {
        match self {
            GradeScaleType::IHK => "IHK",
            GradeScaleType::TECHNIKER => "TECHNIKER",
            GradeScaleType::LINEAR => "LINEAR",
            GradeScaleType::Custom(_) => "CUSTOM",
        }
    }

    // return a text representation of the scale
    pub fn key_binding(&self) -> &'static str {
        match self {
            GradeScaleType::IHK => "I",
            GradeScaleType::TECHNIKER => "T",
            GradeScaleType::LINEAR => "L",
            GradeScaleType::Custom(_) => "C",
        }
    }

    // return an associated color
    pub fn color(&self) -> Color {
        match &self {
            GradeScaleType::IHK => Color::Yellow,
            GradeScaleType::TECHNIKER => Color::Blue,
            GradeScaleType::LINEAR => Color::Green,
            GradeScaleType::Custom(_) => Color::LightRed,
        }
    }

    // Check if it is a custom scale
    pub fn is_custom(&self) -> bool {
        matches!(self, GradeScaleType::Custom(_))
    }

    // Convert to a custom scale
    pub fn to_custom(&self) -> GradeScaleType {
        GradeScaleType::Custom(self.values())
    }

    // change a value in the custom scale.
    // the given value will be clamped to 0-1.
    pub fn change(&mut self, index: usize, value: f64) {
        // only if Custom scale
        if let GradeScaleType::Custom(values) = self {
            // check if index is not out of bound
            if (0..=5).contains(&index) {
                values[index].1 = (value).clamp(0.0, 1.0); // Ensure no overflow
            }
        }
    }
}

#[derive(Debug)]
pub enum GradingError {
    InvalidGrade(u8),
    InvalidPoints(f64),
}

#[derive(Debug, Default)]
pub struct GradingScale {
    scale_type: GradeScaleType,
    total_points: f64,
    thresholds: BTreeMap<Grade, f64>,
    use_half_points: bool, // Flag to indicate if half-points are allowed
}

impl GradingScale {
    pub fn from_type(scale_type: GradeScaleType, max_points: f64) -> Result<Self, GradingError> {
        if max_points <= 0.0 {
            return Err(GradingError::InvalidPoints(max_points));
        }

        let thresholds = GradingScale::calculate_thresholds(&scale_type, max_points)?;

        info!("INIT GradingScale of type {}", scale_type.text());
        Ok(Self {
            scale_type,
            total_points: max_points,
            thresholds,
            use_half_points: false,
        })
    }

    pub fn total_points(&self) -> f64 {
        self.total_points
    }

    pub fn set_total_points(&mut self, total: f64) {
        self.total_points = total;
        self.recalculate();
    }

    // toggle half steps option.
    pub fn toggle_half_points(&mut self) {
        self.use_half_points = !self.use_half_points;
        self.recalculate();
    }

    // returns if half steps are active.
    pub fn is_using_half_points(&self) -> bool {
        self.use_half_points
    }

    pub fn scale_type(&self) -> &GradeScaleType {
        &self.scale_type
    }

    pub fn change_scale_type(&mut self, scale_type: GradeScaleType) {
        self.scale_type = scale_type;
        self.recalculate();
    }

    // recalculates the thresholds
    pub fn recalculate(&mut self) {
        if let Ok(thresholds) =
            GradingScale::calculate_thresholds(&self.scale_type, self.total_points)
        {
            debug!(
                "recalculate ({}) thresholds: {:?}",
                self.scale_type.text(),
                thresholds
            );
            self.thresholds = thresholds;
        }
    }

    // Method to calculate thresholds
    fn calculate_thresholds(
        scale_type: &GradeScaleType,
        max_points: f64,
    ) -> Result<BTreeMap<Grade, f64>, GradingError> {
        let thresholds = scale_type
            .values()
            .iter()
            .map(|(grade, pct)| (Grade::try_from(*grade).unwrap(), (pct * max_points).round()))
            .collect();
        Ok(thresholds)
    }

    pub fn increment_points_for_grade(&mut self, grade: Grade) -> Result<(), GradingError> {
        if let Some(points) = self.thresholds.get(&grade) {
            let new_points = if self.is_using_half_points() {
                points + 0.5
            } else {
                points + 1.0
            };
            self.update_points_for_grade(grade, new_points)
        } else {
            Err(GradingError::InvalidGrade(grade.to_number()))
        }
    }

    pub fn decrement_points_for_grade(&mut self, grade: Grade) -> Result<(), GradingError> {
        if let Some(points) = self.thresholds.get(&grade) {
            let new_points = if self.is_using_half_points() {
                points - 0.5
            } else {
                points - 1.0
            };
            self.update_points_for_grade(grade, new_points)
        } else {
            Err(GradingError::InvalidGrade(grade.to_number()))
        }
    }

    // update points for a specific grade
    fn update_points_for_grade(
        &mut self,
        grade: Grade,
        new_points: f64,
    ) -> Result<(), GradingError> {
        debug!("UPDATE Points: {}", new_points);
        if let Some(points) = self.thresholds.get_mut(&grade) {
            // change scale type to custom if points where changed
            if !self.scale_type.is_custom() {
                self.scale_type = self.scale_type.to_custom();
            }
            *points = new_points.round();
            Ok(())
        } else {
            Err(GradingError::InvalidPoints(new_points))
        }
    }

    pub fn grade_for_points(&self, points: f64) -> Option<Grade> {
        self.thresholds
            .iter()
            .find(|(_, &pts)| points > pts)
            .map(|(grade, _)| *grade)
    }

    pub fn percentage_for_points(points: f64, total: f64) -> f64 {
        round_dp(points / total, 2)
    }

    pub fn to_grading_scale_table_data(&self) -> Vec<GradingScaleTableRowData> {
        self.thresholds
            .iter()
            .map(|(grade, &min)| {
                let pct = GradingScale::percentage_for_points(min, self.total_points);

                let max = if let Some(better_grade) = grade.next_better() {
                    *self.thresholds.get(&better_grade).unwrap() - 1.0 // todo: does not take half points into account
                } else {
                    self.total_points
                };

                GradingScaleTableRowData::new(grade.to_number(), min, max, pct)
            })
            .collect()
    }
}

/// helper function to round a number to given decimal places.
pub fn round_dp(value: f64, dp: usize) -> f64 {
    let x = 10u32.pow(dp as u32) as f64;
    (value * x).round() / x
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter, EnumString, Ord, PartialOrd)]
pub enum Grade {
    VeryGood,     // 1
    Good,         // 2
    Satisfactory, // 3
    Sufficient,   // 4
    Poor,         // 5
    Fail,         // 6
}

impl Grade {
    pub fn to_number(self) -> u8 {
        match self {
            Grade::VeryGood => 1,
            Grade::Good => 2,
            Grade::Satisfactory => 3,
            Grade::Sufficient => 4,
            Grade::Poor => 5,
            Grade::Fail => 6,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Grade::VeryGood => "Very Good",
            Grade::Good => "Good",
            Grade::Satisfactory => "Satisfactory",
            Grade::Sufficient => "Sufficient",
            Grade::Poor => "Poor",
            Grade::Fail => "Fail",
        }
    }

    pub fn next_better(self) -> Option<Self> {
        match self {
            Grade::VeryGood => None,
            Grade::Good => Some(Grade::VeryGood),
            Grade::Satisfactory => Some(Grade::Good),
            Grade::Sufficient => Some(Grade::Satisfactory),
            Grade::Poor => Some(Grade::Sufficient),
            Grade::Fail => Some(Grade::Poor),
        }
    }
}

impl TryFrom<u8> for Grade {
    type Error = GradingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Grade::VeryGood),
            2 => Ok(Grade::Good),
            3 => Ok(Grade::Satisfactory),
            4 => Ok(Grade::Sufficient),
            5 => Ok(Grade::Poor),
            6 => Ok(Grade::Fail),
            _ => Err(GradingError::InvalidGrade(value)),
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_number())
    }
}
