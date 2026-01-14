use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::config::Config;
use crate::creature::Creature;
use crate::evolution::{random_population, EvolutionManager};
use crate::frame_dump::dump_frame;
use crate::metrics::{compute_metrics, GenerationMetrics};
use crate::world::World;

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub metrics: Vec<GenerationMetrics>,
}

pub fn run_simulation(config: &Config) -> SimulationResult {
    let mut rng = StdRng::seed_from_u64(config.seed);
    run_simulation_with_rng(config, &mut rng)
}

pub fn run_simulation_with_rng(config: &Config, rng: &mut StdRng) -> SimulationResult {
    let mut metrics = Vec::with_capacity(config.generations);
    let mut population = random_population(config.population, config.nn_sizes(), rng);
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
                let (dx, dy) = world.nearest_food(creature.x, creature.y);
                let inputs = creature.perceive(dx, dy, config.max_energy, rng);
                let action = creature.decide(&inputs);
                creature.act(action, world.width, world.height, config.move_cost);
                if creature.alive {
                    if world.try_eat_food(creature.x, creature.y, rng) {
                        creature.energy = (creature.energy + config.food_energy)
                            .min(config.max_energy);
                        food_eaten_total += 1;
                    }
                }
            }
            if config.dump_frames && step % config.frame_every == 0 {
                let _ = dump_frame(&config.frames_dir, gen, step, &world, &population);
            }
            if !alive_any {
                break;
            }
        }

        let gen_metrics = compute_metrics(gen, &population, food_eaten_total);
        metrics.push(gen_metrics);

        if gen + 1 < config.generations {
            population = evolution.next_generation(&population, config.nn_sizes(), rng);
        }
        if config.progress > 0 && (gen + 1) % config.progress == 0 {
            println!("Generation {} complete", gen + 1);
        }
    }

    SimulationResult { metrics }
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
    }
}
