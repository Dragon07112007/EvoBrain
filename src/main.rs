use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

use clap::Parser;

use evobrain::config::Config;
use evobrain::metrics::GenerationMetrics;
use evobrain::simulation::run_simulation;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();
    let result = run_simulation(&config);
    write_csv(&config.out, &result.metrics)?;
    write_metadata(&config.run_metadata, &config, &result.metrics)?;
    Ok(())
}

fn write_csv(path: &str, metrics: &[GenerationMetrics]) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut wtr = csv::Writer::from_writer(writer);
    for metric in metrics {
        wtr.serialize(metric)?;
    }
    wtr.flush()?;
    Ok(())
}

fn write_metadata(
    path: &str,
    config: &Config,
    metrics: &[GenerationMetrics],
) -> Result<(), Box<dyn Error>> {
    #[derive(serde::Serialize)]
    struct RunMetadata<'a> {
        config: &'a Config,
        generations: usize,
        nn_sizes: (usize, usize, usize),
    }

    let metadata = RunMetadata {
        config,
        generations: metrics.len(),
        nn_sizes: config.nn_sizes(),
    };
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &metadata)?;
    writer.write_all(b"\n")?;
    Ok(())
}
