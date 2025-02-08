use csv::Error;

use crate::grade::Grade;

#[derive(Debug, Default)]
pub struct CsvExporter {
    file: String,
}

impl CsvExporter{
    pub fn new(output: &str) -> Self {
        CsvExporter { file: output.into()}
    }

    pub fn export(&self, data: &Vec<Grade>) -> Result<(), Error> {
        let mut wtr = csv::Writer::from_path(&self.file)?;

        for grade in data.into_iter() {
            wtr.serialize((
                grade.value().to_string(), 
                grade.min().to_string(),
                grade.max().to_string(),
                grade.pct(data[0].max())
            ))?
        }
        wtr.flush()?;

        // println!("Writing result to file {}", self.file);
        Ok(())
    }
}


