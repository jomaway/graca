use std::convert::TryFrom;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Grade(u8);

#[derive(Debug)]
pub enum GradeError {
    InvalidGrade(u8),
}

impl Grade {
    pub fn new(value: u8) -> Result<Self, GradeError> {
        if (1..=6).contains(&value) {
            Ok(Self(value))
        } else {
            Err(GradeError::InvalidGrade(value))
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }

    pub fn next_worse(self) -> Option<Self> {
        Grade::new(self.value() + 1).ok()
    }

    pub fn next_better(self) -> Option<Self> {
        Grade::new(self.value() - 1).ok()
    }
}

impl TryFrom<u8> for Grade {
    type Error = GradeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Grade::new(value)
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// helper function to round a number to given decimal places.
pub fn round_dp(value: f64, dp: usize) -> f64 {
    let x = 10u32.pow(dp as u32) as f64;
    (value * x).round() / x
}
