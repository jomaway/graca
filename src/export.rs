use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use csv::Error as CsvError;
use directories::UserDirs;
use rust_xlsxwriter::{Format, Workbook, XlsxError};

use crate::ui::grading_scale_table::GradingScaleTableRowData;

#[derive(Debug, Clone, PartialEq)]
pub struct ExportError {
    details: String,
}

impl ExportError {
    pub fn msg(&self) -> String {
        self.details.clone()
    }
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

impl From<toml::ser::Error> for ExportError {
    fn from(value: toml::ser::Error) -> Self {
        ExportError {
            details: value.to_string(),
        }
    }
}

pub trait Exporter {
    fn export(path: &Path, data: &Vec<GradingScaleTableRowData>) -> Result<(), ExportError>;
}

pub struct CsvExporter;
pub struct TomlExporter;
pub struct XlsxExporter;

impl Exporter for CsvExporter {
    fn export(path: &Path, data: &Vec<GradingScaleTableRowData>) -> Result<(), ExportError> {
        let mut wtr = csv::Writer::from_path(path)?;

        for data_row in data.into_iter() {
            wtr.serialize(data_row.as_str_array())?
        }
        wtr.flush()?;

        // println!("Writing result to file {}", self.file);
        Ok(())
    }
}

impl Exporter for TomlExporter {
    fn export(path: &Path, data: &Vec<GradingScaleTableRowData>) -> Result<(), ExportError> {
        let mut dict: HashMap<String, String> = HashMap::new();
        for [grade, min, max, pct] in data.iter().map(|row| row.as_str_array()) {
            dict.insert(grade, format!("({},{},{})", min, max, pct));
        }

        let toml_string = toml::to_string_pretty(&dict)?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}

impl Exporter for XlsxExporter {
    fn export(path: &Path, data: &Vec<GradingScaleTableRowData>) -> Result<(), ExportError> {
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

        for (idx, row_data) in data.iter().map(|row| row.as_str_array()).enumerate() {
            let idx = idx as u32;
            worksheet.write(idx + 1, 0, row_data[0].clone())?;
            worksheet.write(idx + 1, 1, row_data[1].clone())?;
            worksheet.write(idx + 1, 2, row_data[2].clone())?;
            worksheet.write(idx + 1, 3, row_data[3].clone())?;
        }

        workbook.save(path)?;

        Ok(())
    }
}

pub fn export(path: &Path, data: &Vec<GradingScaleTableRowData>) -> Result<(), ExportError> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("csv") => Ok(CsvExporter::export(path, data)?),
        Some("toml") => Ok(TomlExporter::export(path, data)?),
        Some("xlsx") => Ok(XlsxExporter::export(path, data)?),
        _ => Err(ExportError {
            details: "File type not supported.".to_string(),
        }),
    }
}

pub fn resolve_path(user_input: &str) -> Option<PathBuf> {
    let path = PathBuf::from(user_input);

    if path.is_absolute() {
        Some(path)
    } else if user_input.starts_with("~") {
        expand_home(&path)
    } else {
        std::env::current_dir().ok().map(|cwd| cwd.join(path))
    }
}

// Takes a string and appends it to the home directory
fn expand_home(path: &PathBuf) -> Option<PathBuf> {
    if let Some(mut home_path) = UserDirs::new().and_then(|u| Some(u.home_dir().to_path_buf())) {
        home_path.push(path);
        Some(home_path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_home() {
        let relative_path = PathBuf::from("test_folder/test_file.txt");
        if let Some(expanded) = expand_home(&relative_path) {
            assert!(expanded.starts_with(UserDirs::new().unwrap().home_dir()));
            assert!(expanded.ends_with("test_folder/test_file.txt"));
        } else {
            panic!("Failed to expand home directory");
        }
    }

    #[test]
    fn test_resolve_path() {
        let home_dir = UserDirs::new().unwrap().home_dir().to_path_buf();

        assert!(resolve_path("/absolute/path").unwrap().is_absolute());
        assert!(resolve_path("relative/path").unwrap().is_absolute());
        assert!(resolve_path("~/home_path").unwrap().starts_with(&home_dir));
    }

    #[test]
    fn test_export() {
        let data = vec![];
        assert_eq!(export(&PathBuf::from("test.csv"), &data), Ok(()));
        assert_eq!(export(&PathBuf::from("test.xlsx"), &data), Ok(()));
        assert_eq!(
            export(&PathBuf::from("test.txt"), &data),
            Err(ExportError {
                details: "File type not supported.".to_string(),
            })
        )
    }
}
