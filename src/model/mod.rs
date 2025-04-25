pub mod scale;
pub mod students;

use std::{collections::HashMap, path::Path};

use crate::{
    action::ModelAction,
    ui::{
        exam_result_table::ExamResultTableRowData, grading_scale_table::GradingScaleTableRowData,
    },
};
use scale::{round_dp, Grade, GradeScaleType, GradingScale};
use students::StudentList;

#[derive(Debug, Default)]
pub struct Model {
    pub scale: GradingScale,
    student_list: StudentList,
}

impl Model {
    pub fn new() -> Self {
        let scale = GradingScale::from_type(GradeScaleType::IHK, 100.0).unwrap(); // todo! better error handling.

        Self {
            scale,
            student_list: StudentList::default(),
        }
    }

    pub fn load_student_data(&mut self, path: &Path) -> std::io::Result<()> {
        self.student_list = StudentList::from_csv_file(path)?;
        Ok(())
    }

    pub fn update(&mut self, action: ModelAction) {
        match action {
            ModelAction::IncrementThreshold(grade) => {
                if let Ok(grade) = Grade::try_from(grade) {
                    self.scale
                        .increment_points_for_grade(grade)
                        .expect("Grade not found");
                }
            }
            ModelAction::DecrementThreshold(grade) => {
                if let Ok(grade) = Grade::try_from(grade) {
                    self.scale
                        .decrement_points_for_grade(grade)
                        .expect("Grade not found");
                }
            }
            ModelAction::SetMaxPoints(points) => self.scale.set_max_points(points as f64),
            ModelAction::SetScale(value) => {
                if let Ok(scale_type) = GradeScaleType::try_from(value) {
                    self.scale.change_scale_type(scale_type);
                }
            }
            ModelAction::ToggleHalfPoints => {
                self.scale.toggle_half_points();
            }
            ModelAction::IncrementMaxPoints => {
                self.scale.set_max_points(self.scale.max_points() + 1.0);
            }
            ModelAction::DecrementMaxPoints => {
                self.scale.set_max_points(self.scale.max_points() - 1.0);
            }
            ModelAction::IncrementStudentPoints(name) => {
                if let Some(student) = self.student_list.get_student_mut(&name) {
                    let new_value = match self.scale.is_using_half_points() {
                        true => student.total() + 0.5,
                        false => student.total() + 1.0,
                    };

                    if new_value <= self.scale.max_points() {
                        student.update_points(new_value);
                    }
                }
            }
            ModelAction::DecrementStudentPoints(name) => {
                if let Some(student) = self.student_list.get_student_mut(&name) {
                    let new_value = match self.scale.is_using_half_points() {
                        true => student.total() - 0.5,
                        false => student.total() - 1.0,
                    };

                    if new_value <= self.scale.max_points() {
                        student.update_points(new_value);
                    }
                }
            }
        }
    }

    pub fn get_scale_data(&self) -> Vec<GradingScaleTableRowData> {
        let mut last_min = self.scale.max_points();
        self.scale
            .thresholds()
            .iter()
            .map(|(grade, &min)| {
                let pct = GradingScale::percentage_for_points(min, self.scale.max_points());

                let max = if *grade == Grade::VeryGood {
                    last_min
                } else {
                    match self.scale.is_using_half_points() {
                        true => last_min - 0.5,
                        false => last_min - 1.0,
                    }
                };
                last_min = min;

                GradingScaleTableRowData::new(grade.to_number(), min, max, pct)
            })
            .collect()
    }

    pub fn get_class_name(&self) -> &str {
        self.student_list.class_name()
    }

    pub fn get_student_data(&self) -> Vec<ExamResultTableRowData> {
        let mut data = Vec::new();
        for student in self.student_list.iter_students() {
            let points = student.total();
            let row = ExamResultTableRowData::new(
                &student.name,
                points,
                GradingScale::percentage_for_points(points, self.scale.max_points()),
                match self.scale.grade_for_points(points) {
                    Some(grade) => grade.to_number(),
                    None => 0,
                },
            );
            data.push(row);
        }
        data
    }

    pub fn grade_distribution(&self) -> HashMap<u8, usize> {
        let mut counts = HashMap::new();
        for student in self.student_list.iter_students() {
            let grade = student.grade(&self.scale); // returns a u8
            counts
                .entry(grade.to_number())
                .and_modify(|counter| *counter += 1)
                .or_insert(0);
        }

        counts
    }

    pub fn grade_average(&self) -> f64 {
        let mut grades_weighted = 0;
        let mut total_count = 0;

        for (grade, count) in self.grade_distribution() {
            if (1..=6).contains(&grade) {
                total_count += count;
                grades_weighted += (grade as usize) * count;
            }
        }

        round_dp(grades_weighted as f64 / total_count as f64, 2)
    }
}
