use core::fmt;
use std::error::Error;

use csv::Error as CsvError;
use rust_xlsxwriter::{Format, Workbook, XlsxError};

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

// for XlsxError
impl From<XlsxError> for ExportError {
    fn from(value: XlsxError) -> Self {
        ExportError {
            details: value.to_string(),
        }
    }
}

pub trait Exporter {
    fn export(&self, data: &Vec<Grade>) -> Result<(), ExportError>;
}

#[derive(Debug, Default)]
pub struct CsvExporter {
    path: String,
}

impl CsvExporter {
    pub fn new(output: &str) -> Self {
        CsvExporter {
            path: output.into(),
        }
    }
}

impl Exporter for CsvExporter {
    fn export(&self, data: &Vec<Grade>) -> Result<(), ExportError> {
        let mut wtr = csv::Writer::from_path(format!("{}.csv", self.path))?;

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



#[derive(Debug, Default)]
pub struct ExcelExporter {
    path: String,
}

impl ExcelExporter {
    pub fn new(output: &str) -> Self {
        ExcelExporter {
            path: output.into(),
        }
    }
}

impl Exporter for ExcelExporter {
    fn export(&self, data: &Vec<Grade>) -> Result<(), ExportError> {
        // Create a new Excel file object.
        let mut workbook = Workbook::new();

        // Add a bold format to use to highlight cells.
        let bold = Format::new().set_bold();

        // Add a worksheet to the workbook.
        let worksheet = workbook.add_worksheet();

        // Write a string to cell (0, 0) = A1.
        worksheet.write_with_format(0, 0, "Note", &bold)?;
        worksheet.write_with_format(0, 1, "min", &bold)?;
        worksheet.write_with_format(0, 2, "max", &bold)?;
        worksheet.write_with_format(0, 3, "%", &bold)?;

        for (idx,grade) in data.iter().enumerate() { 
            let idx = idx as u32; 
            worksheet.write(idx+1,0, grade.value().to_string())?;
            worksheet.write(idx+1,1, grade.min().to_string())?;
            worksheet.write(idx+1,2, grade.max().to_string())?;
            worksheet.write(idx+1,3, grade.pct(data[0].max()).to_string())?;
        }

        workbook.save(format!("{}.xlsx", self.path))?;

        Ok(())
    }
}
