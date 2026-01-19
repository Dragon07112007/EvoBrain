use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug, Clone, Serialize)]
#[command(author, version, about = "Headless evolutionary simulation")]
pub struct Config {
    #[arg(long, default_value_t = 100)]
    pub generations: usize,
    #[arg(long, default_value_t = 200)]
    pub population: usize,
    #[arg(long, default_value_t = 50)]
    pub width: usize,
    #[arg(long, default_value_t = 50)]
    pub height: usize,
    #[arg(long, default_value_t = 100)]
    pub food: usize,
    #[arg(long, default_value_t = 1000)]
    pub max_steps: usize,
    #[arg(long, default_value_t = 100.0)]
    pub max_energy: f32,
    #[arg(long, default_value_t = 1.0)]
    pub move_cost: f32,
    #[arg(long, default_value_t = 30.0)]
    pub food_energy: f32,
    #[arg(long, default_value_t = 42)]
    pub seed: u64,
    #[arg(long, default_value_t = 4)]
    pub input: usize,
    #[arg(long, default_value_t = 8)]
    pub hidden: usize,
    #[arg(long, default_value_t = 4)]
    pub output: usize,
    #[arg(long, default_value_t = 0.1)]
    pub elite: f32,
    #[arg(long = "mut-rate", default_value_t = 0.05)]
    pub mut_rate: f32,
    #[arg(long = "mut-strength", default_value_t = 0.2)]
    pub mut_strength: f32,
    #[arg(long, default_value = "results.csv")]
    pub out: String,
    #[arg(long, default_value = "run.json")]
    pub run_metadata: String,
    #[arg(long, default_value_t = false)]
    pub dump_frames: bool,
    #[arg(long, default_value_t = 1)]
    pub frame_every: usize,
    #[arg(long, default_value = "PythonFrameConverter/data")]
    pub frames_dir: String,
    #[arg(long, default_value_t = 10)]
    pub progress: usize,
}

impl Config {
    pub fn nn_sizes(&self) -> (usize, usize, usize) {
        (self.input, self.hidden, self.output)
    }
}
