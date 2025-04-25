// use std::collections::HashMap;

// use ratatui::style::Color;
// use serde::Deserialize;
// use strum_macros::EnumIter;

// const IHK_BOUNDARIES: [(u8, f64); 6] = [
//     (1, 0.92),
//     (2, 0.81),
//     (3, 0.67),
//     (4, 0.5),
//     (5, 0.3),
//     (6, 0.0),
// ];

// const TECHNIKER_BOUNDARIES: [(u8, f64); 6] =
//     [(1, 0.85), (2, 0.7), (3, 0.55), (4, 0.4), (5, 0.2), (6, 0.0)];

// const LINEAR_BOUNDARIES: [(u8, f64); 6] = [
//     (1, 0.87),
//     (2, 0.6),
//     (3, 0.47),
//     (4, 0.3),
//     (5, 0.17),
//     (6, 0.0),
// ];

// #[derive(Debug, Default, Clone, Deserialize, EnumIter)]
// pub enum GradeScale {
//     #[default]
//     IHK,
//     TECHNIKER,
//     LINEAR,
//     Custom([(u8, f64); 6]),
// }

// impl GradeScale {
//     // return the boundary values for a scale.
//     pub fn values(&self) -> [(u8, f64); 6] {
//         match self {
//             GradeScale::IHK => IHK_BOUNDARIES,
//             GradeScale::TECHNIKER => TECHNIKER_BOUNDARIES,
//             GradeScale::LINEAR => LINEAR_BOUNDARIES,
//             GradeScale::Custom(values) => *values,
//         }
//     }

//     // return a text representation of the scale
//     pub fn text(&self) -> &'static str {
//         match self {
//             GradeScale::IHK => "IHK",
//             GradeScale::TECHNIKER => "TECHNIKER",
//             GradeScale::LINEAR => "LINEAR",
//             GradeScale::Custom(_) => "CUSTOM",
//         }
//     }

//     // return a text representation of the scale
//     pub fn key_binding(&self) -> &'static str {
//         match self {
//             GradeScale::IHK => "I",
//             GradeScale::TECHNIKER => "T",
//             GradeScale::LINEAR => "L",
//             GradeScale::Custom(_) => "C",
//         }
//     }

//     // return an associated color
//     pub fn color(&self) -> Color {
//         match &self {
//             GradeScale::IHK => Color::Yellow,
//             GradeScale::TECHNIKER => Color::Blue,
//             GradeScale::LINEAR => Color::Green,
//             GradeScale::Custom(_) => Color::LightRed,
//         }
//     }

//     // Check if it is a custom scale
//     pub fn is_custom(&self) -> bool {
//         matches!(self, GradeScale::Custom(_))
//     }

//     // Convert to a custom scale
//     pub fn to_custom(&self) -> GradeScale {
//         GradeScale::Custom(self.values())
//     }

//     // change a value in the custom scale.
//     // the given value will be clamped to 0-1.
//     pub fn change(&mut self, index: usize, value: f64) {
//         // only if Custom scale
//         if let GradeScale::Custom(values) = self {
//             // check if index is not out of bound
//             if (0..=5).contains(&index) {
//                 values[index].1 = (value).clamp(0.0, 1.0); // Ensure no overflow
//             }
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct GradeRange {
//     min: f64,
//     max: f64,
// }

// pub struct Grade {
//     value: u32,
//     range: GradeRange,
// }

// impl Grade {
//     pub fn new(value: u32, min: f64, max: f64) -> Self {
//         Self {
//             value,
//             range: GradeRange { min, max },
//         }
//     }

//     pub fn value(&self) -> u32 {
//         self.value
//     }

//     pub fn min(&self) -> f64 {
//         self.range.min
//     }

//     pub fn max(&self) -> f64 {
//         self.range.max
//     }

//     pub fn pct(&self, total: f64) -> f64 {
//         round_dp(self.range.min / total, 2)
//     }
// }

// // Grading Calculator
// #[derive(Debug, Clone)]
// pub struct GradeCalculator {
//     pub total_points: u32,
//     pub scale: GradeScale,
//     pub half_steps: bool,
//     pub data: HashMap<u32, (f64, f64)>,
// }

// impl Default for GradeCalculator {
//     fn default() -> Self {
//         Self {
//             total_points: 100,
//             scale: GradeScale::IHK,
//             half_steps: false,
//             data: HashMap::new(),
//         }
//     }
// }

// impl GradeCalculator {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     // pub fn total(mut self, points: u32) -> Self {
//     //     self.total_points = points;
//     //     self
//     // }

//     // pub fn scale(mut self, scale: GradeScale) -> Self {
//     //     self.scale = scale;
//     //     self
//     // }

//     pub fn toggle_steps(&mut self) {
//         self.half_steps = !self.half_steps
//     }

//     // get the min points for a grade
//     pub fn min_for(&self, grade: u32) -> Option<f64> {
//         let scale_values = self.scale.values();
//         match scale_values
//             .iter()
//             .find(|&&(x, _)| x == grade as u8)
//             .copied()
//         {
//             Some((_, pct)) => Some(self.calc_points_from_percentage(pct)),
//             None => None,
//         }
//     }

//     fn update_data(&mut self) {
//         let scale_values = self.scale.values();
//         for i in 0..scale_values.len() {
//             let grade = scale_values[i].0;
//             let min_percentage = scale_values[i].1;
//             let max_percentage = if i == 0 {
//                 1.0 // Maximum percentage for grade 1
//             } else {
//                 scale_values[i - 1].1
//             };

//             let min_points = self.calc_points_from_percentage(min_percentage);
//             let max_points = if i == 0 {
//                 self.total_points as f64
//             } else {
//                 let sub = if self.half_steps { 0.5 } else { 1.0 };
//                 self.calc_points_from_percentage(max_percentage) - sub
//             };

//             self.data.insert(grade as u32, (min_points, max_points));
//         }
//     }

//     fn calc_points_from_percentage(&self, pct: f64) -> f64 {
//         (pct * self.total_points as f64).round()
//     }

//     pub fn calc(&self) -> Vec<Grade> {
//         let mut grades = Vec::new();

//         let scale_values = self.scale.values();
//         for i in 0..scale_values.len() {
//             let grade = scale_values[i].0;
//             let min_percentage = scale_values[i].1;
//             let max_percentage = if i == 0 {
//                 1.0 // Maximum percentage for grade 1
//             } else {
//                 scale_values[i - 1].1
//             };

//             let min_points = self.calc_points_from_percentage(min_percentage);
//             let max_points = if i == 0 {
//                 self.total_points as f64
//             } else {
//                 let sub = if self.half_steps { 0.5 } else { 1.0 };
//                 self.calc_points_from_percentage(max_percentage) - sub
//             };

//             grades.push(Grade::new(grade as u32, min_points, max_points));
//         }

//         grades
//     }
// }

// /// helper function to round a number to given decimal places.
// pub fn round_dp(value: f64, dp: usize) -> f64 {
//     let x = 10u32.pow(dp as u32) as f64;
//     (value * x).round() / x
// }
