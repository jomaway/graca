use std::path::PathBuf;

pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "simple grade point calculator")]
pub struct Args {
    #[arg(
        help = "Path to the courses student list to be opened.",
        required = false
    )]
    pub course: Option<PathBuf>,

    #[arg(
        long,
        help = "Sets the course name. By default, the name is selected based on the courses file name."
    )]
    pub course_name: Option<String>,

    /// max reachable points for the exam.
    #[arg(short, long, default_value_t = 100)]
    pub points: u32,

    #[arg(short, long, default_value_t = String::from("IHK"))]
    pub scale: String,
}
