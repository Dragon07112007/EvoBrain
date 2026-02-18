use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::metrics::selection::parse_gen_selection;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum SelectionMethod {
    Roulette,
    Tournament,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum LoggingMode {
    Full,
    Quick,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum FitnessMode {
    Classic,
    #[value(name = "efficient")]
    EfficientCollector,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum BrainMode {
    Fixed,
    Evolvable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum CrossoverMode {
    None,
    Layer,
    Blend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ArchInherit {
    Fitter,
    Random,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum DistanceMetric {
    Euclidean,
    Manhattan,
}

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
    #[arg(long = "selection", value_enum, default_value_t = SelectionMethod::Roulette)]
    pub selection_method: SelectionMethod,
    #[arg(long, default_value_t = 5)]
    pub tournament_k: u32,
    #[arg(long, default_value_t = 0.1)]
    pub elite: f32,
    #[arg(long = "mut-rate", default_value_t = 0.05)]
    pub mut_rate: f32,
    #[arg(long = "mut-strength", default_value_t = 0.2)]
    pub mut_strength: f32,
    #[arg(long = "fitness", value_enum, default_value_t = FitnessMode::Classic)]
    pub fitness_mode: FitnessMode,
    #[arg(long, default_value_t = 1.0)]
    pub fitness_food_weight: f32,
    #[arg(long, default_value_t = 1.0)]
    pub fitness_efficiency_weight: f32,
    #[arg(long, default_value_t = 0.1)]
    pub fitness_survival_weight: f32,
    #[arg(long, default_value_t = 0.5)]
    pub fitness_idle_weight: f32,
    #[arg(long, default_value_t = 0.2)]
    pub fitness_jitter_weight: f32,
    #[arg(long, default_value_t = 10)]
    pub idle_tolerance: u32,
    #[arg(long = "logging", value_enum, default_value_t = LoggingMode::Full)]
    pub logging_mode: LoggingMode,
    #[arg(long = "log-gens", default_value = "all")]
    pub log_gens: String,
    #[arg(long = "full-log-gens")]
    pub full_log_gens: Option<String>,
    #[arg(long = "full-log-keep", default_value = "10")]
    pub full_log_keep: String,
    #[arg(long = "run-id")]
    pub run_id: Option<String>,
    #[arg(long, default_value_t = 2)]
    pub quick_keep: u32,
    #[arg(long, default_value_t = 0)]
    pub food_vision_radius: u32,
    #[arg(long, value_enum, default_value_t = DistanceMetric::Euclidean)]
    pub distance_metric: DistanceMetric,
    #[arg(long = "brain", value_enum, default_value_t = BrainMode::Fixed)]
    pub brain_mode: BrainMode,
    #[arg(long, default_value_t = 4)]
    pub max_hidden_layers: u32,
    #[arg(long, default_value_t = 4)]
    pub layer_min_neurons: u32,
    #[arg(long, default_value_t = 64)]
    pub layer_max_neurons: u32,
    #[arg(long = "crossover", value_enum, default_value_t = CrossoverMode::None)]
    pub crossover_mode: CrossoverMode,
    #[arg(long = "arch-inherit", value_enum, default_value_t = ArchInherit::Fitter)]
    pub arch_inherit: ArchInherit,
    #[arg(long, default_value = "results.csv")]
    pub out: String,
    #[arg(long, default_value = "run.json")]
    pub run_metadata: String,
    #[arg(long, default_value_t = true)]
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

    pub fn base_layers(&self) -> Vec<usize> {
        vec![self.input, self.hidden, self.output]
    }

    pub fn validate(&self) -> Result<(), String> {
        if matches!(self.selection_method, SelectionMethod::Tournament) && self.tournament_k < 2 {
            return Err(
                "tournament-k must be at least 2 when tournament selection is enabled".to_string(),
            );
        }
        if matches!(self.logging_mode, LoggingMode::Quick) && !matches!(self.quick_keep, 2 | 3) {
            return Err("quick-keep must be 2 or 3 when logging mode is quick".to_string());
        }
        if self.layer_min_neurons == 0 || self.layer_max_neurons == 0 {
            return Err("layer-min-neurons and layer-max-neurons must be positive".to_string());
        }
        if self.layer_min_neurons > self.layer_max_neurons {
            return Err("layer-min-neurons cannot exceed layer-max-neurons".to_string());
        }
        if self.input == 0 || self.output == 0 {
            return Err("input and output layer sizes must be positive".to_string());
        }
        if self.max_hidden_layers == 0 {
            return Err("max-hidden-layers must be at least 1".to_string());
        }
        if let Err(err) = parse_gen_selection(&self.log_gens) {
            return Err(format!("invalid log-gens spec: {err}"));
        }
        if let Some(ref spec) = self.full_log_gens {
            if let Err(err) = parse_gen_selection(spec) {
                return Err(format!("invalid full-log-gens spec: {err}"));
            }
        }
        if let Err(err) = parse_full_log_keep(&self.full_log_keep) {
            return Err(format!("invalid full-log-keep value: {err}"));
        }
        Ok(())
    }
}

pub fn parse_full_log_keep(value: &str) -> Result<Option<usize>, String> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("all") {
        return Ok(None);
    }
    let count = trimmed
        .parse::<usize>()
        .map_err(|_| "must be a positive integer or 'all'".to_string())?;
    if count == 0 {
        return Err("must be greater than 0 or 'all'".to_string());
    }
    Ok(Some(count))
}
