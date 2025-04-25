pub mod scale;
pub mod students;

use std::{collections::HashMap, path::Path};

use scale::{round_dp, GradeScaleType, GradingScale};
use students::{Student, StudentList};

use crate::{
    action::Action,
    ui::{
        exam_result_table::ExamResultTableRowData, grading_scale_table::GradingScaleTableRowData,
    },
};

#[derive(Debug, Default)]
pub struct Model {
    pub scale: GradingScale,
    student_list: Option<StudentList>,
}

impl Model {
    pub fn new() -> Self {
        let scale = GradingScale::from_type(GradeScaleType::IHK, 100.0).unwrap(); // todo! better error handling.

        Self {
            scale,
            student_list: None,
        }
    }

    pub fn load_student_data(&mut self, path: &Path) {
        self.student_list = StudentList::from_csv_file(path).ok();
    }

    pub fn update(&mut self, action: Action) {
        todo!()
    }

    pub fn get_scale_data(&self) -> Vec<GradingScaleTableRowData> {
        let mut last_min = self.scale.max_points();
        self.scale
            .thresholds()
            .iter()
            .map(|(grade, &min)| {
                let pct = GradingScale::percentage_for_points(min, self.scale.max_points());

                let max = last_min;
                last_min = min;

                GradingScaleTableRowData::new(grade.to_number(), min, max, pct)
            })
            .collect()
    }

    pub fn get_class_name(&self) -> String {
        match self.student_list.clone() {
            Some(list) => format!("{}", list),
            None => "".into(),
        }
    }

    pub fn get_student_data(&self) -> Vec<ExamResultTableRowData> {
        match self.student_list.clone() {
            Some(list) => {
                let mut data = Vec::new();
                for student in list.iter_students() {
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
            None => Vec::new(),
        }
    }

    pub fn grade_distribution(&self) -> HashMap<u8, usize> {
        let mut counts = HashMap::new();
        let list = self.student_list.clone().unwrap();
        for student in list.iter_students() {
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
