use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::config::Config;
use crate::metrics::report::GenerationReport;

#[derive(Debug)]
pub struct MetricsWriter {
    run_dir: PathBuf,
    csv_writer: csv::Writer<BufWriter<File>>,
    config_hash: String,
    run_id: String,
    git_commit: Option<String>,
}

#[derive(Debug, Serialize)]
struct RunManifest<'a> {
    run_id: &'a str,
    seed: u64,
    timestamp: String,
    config_hash: &'a str,
    git_commit: Option<&'a str>,
    config: &'a Config,
}

#[derive(Debug, Serialize)]
struct GenerationReportCsvRow {
    generation: u32,
    steps_per_gen: u32,
    population_size: u32,
    fitness_best: f32,
    fitness_mean: f32,
    fitness_median: f32,
    fitness_std: f32,
    fitness_iqr: f32,
    food_eaten_total: u32,
    food_eaten_mean: f32,
    survival_steps_mean: f32,
    reproductions_total: u32,
    params_mean: f32,
    params_median: f32,
    params_best: u32,
    params_std: f32,
    layers_mean: Option<f32>,
    hidden_mean: Option<f32>,
    mutation_rate: f32,
    mutation_sigma: f32,
    crossover_rate: Option<f32>,
    selection_mode: crate::config::SelectionMethod,
    tournament_k: Option<u32>,
    seed: u64,
    run_id: String,
    config_hash: String,
    git_commit: Option<String>,
}

impl From<&GenerationReport> for GenerationReportCsvRow {
    fn from(report: &GenerationReport) -> Self {
        Self {
            generation: report.generation,
            steps_per_gen: report.steps_per_gen,
            population_size: report.population_size,
            fitness_best: report.fitness_best,
            fitness_mean: report.fitness_mean,
            fitness_median: report.fitness_median,
            fitness_std: report.fitness_std,
            fitness_iqr: report.fitness_iqr,
            food_eaten_total: report.food_eaten_total,
            food_eaten_mean: report.food_eaten_mean,
            survival_steps_mean: report.survival_steps_mean,
            reproductions_total: report.reproductions_total,
            params_mean: report.params_mean,
            params_median: report.params_median,
            params_best: report.params_best,
            params_std: report.params_std,
            layers_mean: report.layers_mean,
            hidden_mean: report.hidden_mean,
            mutation_rate: report.mutation_rate,
            mutation_sigma: report.mutation_sigma,
            crossover_rate: report.crossover_rate,
            selection_mode: report.selection_mode,
            tournament_k: report.tournament_k,
            seed: report.seed,
            run_id: report.run_id.clone(),
            config_hash: report.config_hash.clone(),
            git_commit: report.git_commit.clone(),
        }
    }
}

impl MetricsWriter {
    pub fn new(config: &Config, run_id: String) -> Result<Self, Box<dyn Error>> {
        let config_hash = hash_config(config);
        let git_commit = get_git_commit();
        let run_dir = PathBuf::from("runs").join(&run_id);
        create_dir_all(&run_dir)?;
        write_manifest(&run_dir, &run_id, config, &config_hash, git_commit.as_deref())?;
        let csv_path = run_dir.join("generations.csv");
        let file_exists = csv_path.exists();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&csv_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = csv::WriterBuilder::new()
            .has_headers(!file_exists)
            .from_writer(writer);
        if !file_exists {
            csv_writer.flush()?;
        }
        Ok(Self {
            run_dir,
            csv_writer,
            config_hash,
            run_id,
            git_commit,
        })
    }

    pub fn write_generation(&mut self, report: &GenerationReport) -> Result<(), Box<dyn Error>> {
        let json_path = self
            .run_dir
            .join(format!("gen_{:04}.json", report.generation));
        let file = File::create(json_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, report)?;
        writer.write_all(b"\n")?;

        let row = GenerationReportCsvRow::from(report);
        self.csv_writer.serialize(row)?;
        self.csv_writer.flush()?;
        Ok(())
    }

    pub fn config_hash(&self) -> &str {
        &self.config_hash
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    pub fn git_commit(&self) -> Option<&str> {
        self.git_commit.as_deref()
    }
}

pub fn default_run_id(seed: u64) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{timestamp}-{seed}")
}

fn write_manifest(
    run_dir: &Path,
    run_id: &str,
    config: &Config,
    config_hash: &str,
    git_commit: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    let manifest = RunManifest {
        run_id,
        seed: config.seed,
        timestamp,
        config_hash,
        git_commit,
        config,
    };
    let path = run_dir.join("manifest.json");
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &manifest)?;
    writer.write_all(b"\n")?;
    Ok(())
}

fn hash_config(config: &Config) -> String {
    let payload = serde_json::to_vec(config).unwrap_or_default();
    let hash = fnv1a_64(&payload);
    format!("{hash:016x}")
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;
    let mut hash = OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn get_git_commit() -> Option<String> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if commit.is_empty() {
        None
    } else {
        Some(commit)
    }
}
