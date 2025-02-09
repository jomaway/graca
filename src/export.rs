use core::fmt;
use std::error::Error;

use csv::Error as CsvError;

use crate::grade::Grade;

#[derive(Debug, Clone, PartialEq)]
pub struct ExportError {
    details: String,
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ExportError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<std::io::Error> for ExportError {
    fn from(error: std::io::Error) -> Self {
        ExportError {
            details: error.to_string(),
        }
    }
}

// for the CsvExporter
impl From<CsvError> for ExportError {
    fn from(error: CsvError) -> Self {
        ExportError {
            details: error.to_string(),
        }
    }
}

pub trait Exporter {
    fn export(&self, data: &Vec<Grade>) -> Result<(), ExportError>;
}

#[derive(Debug, Default)]
pub struct CsvExporter {
    file: String,
}

impl CsvExporter {
    pub fn new(output: &str) -> Self {
        CsvExporter {
            file: output.into(),
        }
    }
}

impl Exporter for CsvExporter {
    fn export(&self, data: &Vec<Grade>) -> Result<(), ExportError> {
        let mut wtr = csv::Writer::from_path(&self.file)?;

        for grade in data.into_iter() {
            wtr.serialize((
                grade.value().to_string(),
                grade.min().to_string(),
                grade.max().to_string(),
                grade.pct(data[0].max()),
            ))?
        }
        wtr.flush()?;

        // println!("Writing result to file {}", self.file);
        Ok(())
    }
}
