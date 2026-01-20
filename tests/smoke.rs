use std::fs;
use std::io::Read;
use std::path::PathBuf;

use evobrain::config::Config;
use evobrain::genome::{genome_size, Genome};
use evobrain::simulation::run_simulation;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let unique = format!("{}_{}", name, std::process::id());
    path.push(unique);
    path
}

#[test]
fn smoke_run_and_csv() {
    let csv_path = temp_path("evobrain_results.csv");
    let run_path = temp_path("evobrain_run.json");

    let config = Config {
        generations: 3,
        population: 10,
        width: 10,
        height: 10,
        food: 10,
        max_steps: 50,
        max_energy: 20.0,
        move_cost: 1.0,
        food_energy: 5.0,
        seed: 123,
        input: 4,
        hidden: 6,
        output: 4,
        selection_method: evobrain::config::SelectionMethod::Roulette,
        tournament_k: 5,
        elite: 0.2,
        mut_rate: 0.1,
        mut_strength: 0.3,
        fitness_mode: evobrain::config::FitnessMode::Classic,
        fitness_food_weight: 1.0,
        fitness_efficiency_weight: 1.0,
        fitness_survival_weight: 0.1,
        fitness_idle_weight: 0.5,
        fitness_jitter_weight: 0.2,
        idle_tolerance: 10,
        logging_mode: evobrain::config::LoggingMode::Full,
        log_gens: "none".to_string(),
        full_log_gens: None,
        run_id: None,
        quick_keep: 2,
        food_vision_radius: 0,
        distance_metric: evobrain::config::DistanceMetric::Euclidean,
        brain_mode: evobrain::config::BrainMode::Fixed,
        max_hidden_layers: 4,
        layer_min_neurons: 4,
        layer_max_neurons: 64,
        crossover_mode: evobrain::config::CrossoverMode::None,
        arch_inherit: evobrain::config::ArchInherit::Fitter,
        out: csv_path.to_string_lossy().to_string(),
        run_metadata: run_path.to_string_lossy().to_string(),
        dump_frames: false,
        frame_every: 10,
        frames_dir: "frames".to_string(),
        progress: 0,
    };

    let result = run_simulation(&config);
    assert_eq!(result.metrics.len(), 3);
    assert_eq!(result.total_generations, 3);

    let mut wtr = csv::Writer::from_path(&config.out).expect("create csv");
    for metric in &result.metrics {
        wtr.serialize(metric).expect("write metric");
    }
    wtr.flush().expect("flush csv");

    let mut content = String::new();
    fs::File::open(&config.out)
        .expect("open csv")
        .read_to_string(&mut content)
        .expect("read csv");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 4, "header + 3 rows");

    let _ = fs::remove_file(&config.out);
}

#[test]
fn determinism_same_seed() {
    let config = Config {
        generations: 3,
        population: 12,
        width: 8,
        height: 8,
        food: 6,
        max_steps: 30,
        max_energy: 15.0,
        move_cost: 1.0,
        food_energy: 4.0,
        seed: 999,
        input: 4,
        hidden: 5,
        output: 4,
        selection_method: evobrain::config::SelectionMethod::Roulette,
        tournament_k: 5,
        elite: 0.2,
        mut_rate: 0.1,
        mut_strength: 0.2,
        fitness_mode: evobrain::config::FitnessMode::Classic,
        fitness_food_weight: 1.0,
        fitness_efficiency_weight: 1.0,
        fitness_survival_weight: 0.1,
        fitness_idle_weight: 0.5,
        fitness_jitter_weight: 0.2,
        idle_tolerance: 10,
        logging_mode: evobrain::config::LoggingMode::Full,
        log_gens: "none".to_string(),
        full_log_gens: None,
        run_id: None,
        quick_keep: 2,
        food_vision_radius: 0,
        distance_metric: evobrain::config::DistanceMetric::Euclidean,
        brain_mode: evobrain::config::BrainMode::Fixed,
        max_hidden_layers: 4,
        layer_min_neurons: 4,
        layer_max_neurons: 64,
        crossover_mode: evobrain::config::CrossoverMode::None,
        arch_inherit: evobrain::config::ArchInherit::Fitter,
        out: "unused.csv".to_string(),
        run_metadata: "unused.json".to_string(),
        dump_frames: false,
        frame_every: 10,
        frames_dir: "frames".to_string(),
        progress: 0,
    };

    let result_a = run_simulation(&config);
    let result_b = run_simulation(&config);
    assert_eq!(result_a.metrics, result_b.metrics);
}

#[test]
fn genome_size_and_mutation() {
    let size = genome_size(4, 5, 4);
    assert_eq!(size, (4 + 1) * 5 + (5 + 1) * 4);

    let mut rng = StdRng::seed_from_u64(42);
    let mut genome = Genome::random(vec![4, 2, 4], &mut rng);
    let original = genome.weights.clone();
    genome.mutate(1.0, 0.5, &mut rng);
    assert!(genome
        .weights
        .iter()
        .zip(original.iter())
        .any(|(a, b)| a != b));
}
