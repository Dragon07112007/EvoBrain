use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::config::Config;
use crate::config::LoggingMode;
use crate::creature::Creature;
use crate::evolution::{random_population, EvolutionManager};
use crate::frame_dump::dump_frame;
use crate::metrics::collector::MetricsCollector;
use crate::metrics::report::build_generation_report;
use crate::metrics::selection::parse_gen_selection;
use crate::metrics::writer::{default_run_id, MetricsWriter};
use crate::metrics::{compute_metrics, GenerationMetrics};
use crate::world::World;

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub metrics: Vec<GenerationMetrics>,
    pub total_generations: usize,
}

pub fn run_simulation(config: &Config) -> SimulationResult {
    let mut rng = StdRng::seed_from_u64(config.seed);
    run_simulation_with_rng(config, &mut rng)
}

pub fn run_simulation_with_rng(config: &Config, rng: &mut StdRng) -> SimulationResult {
    let mut metrics = Vec::with_capacity(config.generations);
    let mut head_metric: Option<GenerationMetrics> = None;
    let mut tail_metrics: Vec<GenerationMetrics> = Vec::new();
    let mut population = random_population(config.population, config, rng);
    let log_selection =
        parse_gen_selection(&config.log_gens).expect("log-gens should be validated");
    let full_log_selection = config
        .full_log_gens
        .as_deref()
        .map(parse_gen_selection)
        .transpose()
        .expect("full-log-gens should be validated");
    let run_id = config
        .run_id
        .clone()
        .unwrap_or_else(|| default_run_id(config.seed));
    let mut metrics_writer = if log_selection.is_none() {
        None
    } else {
        match MetricsWriter::new(config, run_id.clone()) {
            Ok(writer) => Some(writer),
            Err(err) => {
                eprintln!("Failed to initialize metrics writer: {err}");
                None
            }
        }
    };
    let mut collector = MetricsCollector::new();
    let evolution = EvolutionManager {
        population_size: config.population,
        elite_fraction: config.elite,
        mutation_rate: config.mut_rate,
        mutation_strength: config.mut_strength,
    };

    for gen in 0..config.generations {
        let mut world = World::new(config.width, config.height, config.food, rng);
        initialize_population(&mut population, &world, config.max_energy, rng);
        let mut food_eaten_total = 0;
        let mut steps_run = 0;
        for step in 0..config.max_steps {
            steps_run = step + 1;
            let mut alive_any = false;
            for creature in &mut population {
                if !creature.alive {
                    continue;
                }
                alive_any = true;
                let (dx, dy) = if config.food_vision_radius == 0 {
                    world.nearest_food(creature.x, creature.y)
                } else {
                    world
                        .nearest_food_within(
                            creature.x,
                            creature.y,
                            config.food_vision_radius,
                            config.distance_metric,
                        )
                        .unwrap_or((0.0, 0.0))
                };
                let inputs = creature.perceive(dx, dy, config.max_energy, rng);
                let action = creature.decide(&inputs);
                creature.act(action, world.width, world.height, config.move_cost);
                if creature.alive && world.try_eat_food(creature.x, creature.y, rng) {
                    creature.energy = (creature.energy + config.food_energy).min(config.max_energy);
                    creature.food_collected = creature.food_collected.saturating_add(1);
                    food_eaten_total += 1;
                    collector.on_food_eaten(1);
                }
            }
            if config.dump_frames && step % config.frame_every == 0 && gen == 9999 || gen == 10{
                let _ = dump_frame(&config.frames_dir, gen, step, &world, &population);
            }
            if !alive_any {
                println!("None alive!");
                break;
            }
        }

        let gen_metrics = compute_metrics(gen, &population, food_eaten_total, config);
        if matches!(config.logging_mode, LoggingMode::Full) {
            metrics.push(gen_metrics);
        } else if gen == 0 {
            head_metric = Some(gen_metrics);
        } else {
            tail_metrics.push(gen_metrics);
            let keep = (config.quick_keep.saturating_sub(1)) as usize;
            if tail_metrics.len() > keep {
                tail_metrics.remove(0);
            }
        }

        let mut next_population = None;
        if gen + 1 < config.generations {
            next_population = Some(evolution.next_generation(&population, config, rng, &mut collector));
        }
        if let Some(writer) = metrics_writer.as_mut() {
            let should_log = log_selection.matches(gen as u32);
            if should_log {
                let should_full = matches!(config.logging_mode, LoggingMode::Full)
                    && full_log_selection
                        .as_ref()
                        .map(|selection| selection.matches(gen as u32))
                        .unwrap_or(true);
                let report = build_generation_report(
                    gen as u32,
                    steps_run as u32,
                    &population,
                    &collector,
                    config,
                    writer.run_id(),
                    writer.config_hash(),
                    writer.git_commit(),
                    should_full,
                    10,
                );
                if let Err(err) = writer.write_generation(&report) {
                    eprintln!("Failed to write generation report: {err}");
                }
            }
        }
        if let Some(next_population) = next_population {
            population = next_population;
        }
        collector.reset();
        if config.progress > 0 && (gen + 1) % config.progress == 0 {
            println!("Generation {} complete", gen + 1);
        }
    }

    if !matches!(config.logging_mode, LoggingMode::Full) {
        if let Some(head) = head_metric.take() {
            metrics.push(head);
        }
        metrics.extend(tail_metrics);
    }

    SimulationResult {
        metrics,
        total_generations: config.generations,
    }
}

fn initialize_population(
    population: &mut [Creature],
    world: &World,
    max_energy: f32,
    rng: &mut impl Rng,
) {
    for creature in population {
        creature.x = rng.gen_range(0..world.width.max(1));
        creature.y = rng.gen_range(0..world.height.max(1));
        creature.energy = max_energy;
        creature.age = 0;
        creature.alive = true;
        creature.reset_tracking();
    }
}
