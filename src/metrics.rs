use serde::Serialize;

use crate::config::Config;
use crate::creature::Creature;
use crate::fitness::compute_fitness;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GenerationMetrics {
    pub generation: usize,
    pub avg_fitness: f32,
    pub max_fitness: f32,
    pub avg_age: f32,
    pub survivors: usize,
    pub avg_energy: f32,
    pub food_eaten_total: usize,
}

pub fn compute_metrics(
    generation: usize,
    population: &[Creature],
    food_eaten_total: usize,
    config: &Config,
) -> GenerationMetrics {
    let count = population.len().max(1) as f32;
    let mut sum_fitness = 0.0;
    let mut max_fitness = f32::MIN;
    let mut sum_age = 0.0;
    let mut sum_energy = 0.0;
    let mut survivors = 0;
    for creature in population {
        let fitness = compute_fitness(creature, config);
        sum_fitness += fitness;
        if fitness > max_fitness {
            max_fitness = fitness;
        }
        sum_age += creature.age as f32;
        sum_energy += creature.energy;
        if creature.alive {
            survivors += 1;
        }
    }
    GenerationMetrics {
        generation,
        avg_fitness: sum_fitness / count,
        max_fitness,
        avg_age: sum_age / count,
        survivors,
        avg_energy: sum_energy / count,
        food_eaten_total,
    }
}
