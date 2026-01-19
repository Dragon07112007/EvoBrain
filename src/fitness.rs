use crate::config::{Config, FitnessMode};
use crate::creature::Creature;

const EFFICIENCY_EPS: f32 = 1e-6;

pub fn compute_fitness(creature: &Creature, config: &Config) -> f32 {
    match config.fitness_mode {
        FitnessMode::Classic => creature.fitness_classic(),
        FitnessMode::EfficientCollector => compute_efficient_fitness(creature, config),
    }
}

fn compute_efficient_fitness(creature: &Creature, config: &Config) -> f32 {
    let food_collected = creature.food_collected as f32;
    let efficiency = food_collected / (creature.energy_spent + EFFICIENCY_EPS);
    let survival = creature.survival_steps as f32;
    let idle_penalty = if creature.idle_steps > config.idle_tolerance {
        (creature.idle_steps - config.idle_tolerance) as f32
    } else {
        0.0
    };
    let jitter_penalty = creature.jitter_score as f32;
    config.fitness_food_weight * food_collected
        + config.fitness_efficiency_weight * efficiency
        + config.fitness_survival_weight * survival
        - config.fitness_idle_weight * idle_penalty
        - config.fitness_jitter_weight * jitter_penalty
}
