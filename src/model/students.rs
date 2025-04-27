use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::scale::{Grade, GradingScale};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Student {
    pub name: String,
    points: f64, // todo! change to Vec of points later
}

impl Student {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            points: 0.0,
        }
    }

    pub fn with_points(mut self, points: f64) -> Self {
        self.update_points(points);
        self
    }

    pub fn update_points(&mut self, new_value: f64) {
        self.points = new_value;
    }

    // return total points for a student.
    pub fn total(&self) -> f64 {
        self.points
    }

    pub fn grade(&self, scale: &GradingScale) -> Grade {
        scale.grade_for_points(self.points).unwrap_or(Grade::Fail)
    }
}

#[derive(Debug, Default, Clone)]
pub struct StudentList {
    course: String,
    students: Vec<Student>,
}

impl StudentList {
    pub fn from_csv_file(path: &Path) -> io::Result<Self> {
        // Extract metadata from filename
        let course_name = path
            .file_stem()
            .and_then(|f| f.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid filename"))?;

        debug!("Try to open '{:?}'", path);
        let mut reader = csv::Reader::from_path(path)?;

        let mut students = Vec::new();

        for result in reader.deserialize() {
            let student: Student =
                result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            students.push(student);
        }

        Ok(StudentList {
            course: course_name.to_string(),
            students,
        })
    }

    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let mut writer = csv::Writer::from_path(path)?;

        for student in self.iter_students() {
            writer.serialize(student)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn class_name(&self) -> &str {
        &self.course
    }

    pub fn iter_students(&self) -> impl Iterator<Item = &Student> {
        self.students.iter()
    }

    pub fn iter_students_mut(&mut self) -> impl Iterator<Item = &mut Student> {
        self.students.iter_mut()
    }

    pub fn get_student(&self, name: &str) -> Option<&Student> {
        self.students.iter().find(|s| s.name == name)
    }

    pub fn get_student_mut(&mut self, name: &str) -> Option<&mut Student> {
        self.students.iter_mut().find(|s| s.name == name)
    }
}

impl std::fmt::Display for StudentList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.course)
    }
}
