pub use clap::Parser;


#[derive(Parser, Debug)]
#[command(version, about = "simple grade point calculator")]
pub struct Args {
  /// max reachable points for the exam.
  #[arg(short, long, default_value_t = 100)]
  pub points: u32,
}

