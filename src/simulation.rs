use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::config::Config;
use crate::config::LoggingMode;
use crate::creature::Creature;
use crate::evolution::{random_population, EvolutionManager};
use crate::frame_dump::dump_frame;
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
        for step in 0..config.max_steps {
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

        if gen + 1 < config.generations {
            population = evolution.next_generation(&population, config, rng);
        }
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
