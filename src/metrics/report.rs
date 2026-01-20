use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::config::{Config, SelectionMethod};
use crate::creature::Creature;
use crate::fitness::compute_fitness;
use crate::metrics::collector::MetricsCollector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualSummary {
    pub id: u64,
    pub fitness: f32,
    pub food_eaten: u32,
    pub survival_steps: u32,
    pub params: u32,
    pub layers: Option<u32>,
    pub hidden: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationReport {
    pub generation: u32,
    pub steps_per_gen: u32,
    pub population_size: u32,
    pub fitness_best: f32,
    pub fitness_mean: f32,
    pub fitness_median: f32,
    pub fitness_std: f32,
    pub fitness_iqr: f32,
    pub food_eaten_total: u32,
    pub food_eaten_mean: f32,
    pub survival_steps_mean: f32,
    pub reproductions_total: u32,
    pub params_mean: f32,
    pub params_median: f32,
    pub params_best: u32,
    pub params_std: f32,
    pub layers_mean: Option<f32>,
    pub hidden_mean: Option<f32>,
    pub mutation_rate: f32,
    pub mutation_sigma: f32,
    pub crossover_rate: Option<f32>,
    pub selection_mode: SelectionMethod,
    pub tournament_k: Option<u32>,
    pub seed: u64,
    pub run_id: String,
    pub config_hash: String,
    pub git_commit: Option<String>,
    pub individuals: Option<Vec<IndividualSummary>>,
}

pub fn build_generation_report(
    generation: u32,
    steps_per_gen: u32,
    population: &[Creature],
    collector: &MetricsCollector,
    config: &Config,
    run_id: &str,
    config_hash: &str,
    git_commit: Option<&str>,
    include_individuals: bool,
    top_n: usize,
) -> GenerationReport {
    let mut fitness_values = Vec::with_capacity(population.len());
    let mut params_values = Vec::with_capacity(population.len());
    let mut layers_values = Vec::with_capacity(population.len());
    let mut hidden_values = Vec::with_capacity(population.len());
    let mut survival_sum = 0.0;
    let mut best_fitness = f32::MIN;
    let mut params_best = 0;
    let mut individuals = if include_individuals {
        Some(Vec::with_capacity(population.len()))
    } else {
        None
    };

    for (idx, creature) in population.iter().enumerate() {
        let fitness = compute_fitness(creature, config);
        let params = creature.brain.param_count();
        let layers = creature.brain.layer_count();
        let hidden = creature.brain.hidden_count();
        fitness_values.push(fitness);
        params_values.push(params as f32);
        layers_values.push(layers as f32);
        hidden_values.push(hidden as f32);
        survival_sum += creature.survival_steps as f32;
        if fitness > best_fitness {
            best_fitness = fitness;
            params_best = params;
        }
        if let Some(ref mut list) = individuals {
            list.push(IndividualSummary {
                id: idx as u64,
                fitness,
                food_eaten: creature.food_collected,
                survival_steps: creature.survival_steps,
                params,
                layers: Some(layers),
                hidden: Some(hidden),
            });
        }
    }

    let population_size = population.len() as u32;
    let fitness_mean = mean(&fitness_values);
    let fitness_median = median(&fitness_values);
    let fitness_std = std_dev(&fitness_values, fitness_mean);
    let fitness_iqr = iqr(&fitness_values);
    let params_mean = mean(&params_values);
    let params_median = median(&params_values);
    let params_std = std_dev(&params_values, params_mean);
    let layers_mean = mean_option(&layers_values);
    let hidden_mean = mean_option(&hidden_values);
    let food_eaten_total = collector.food_eaten_total();
    let food_eaten_mean = if population_size > 0 {
        food_eaten_total as f32 / population_size as f32
    } else {
        0.0
    };
    let survival_steps_mean = if population_size > 0 {
        survival_sum / population_size as f32
    } else {
        0.0
    };

    let individuals = individuals.map(|mut list| {
        list.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(Ordering::Equal)
        });
        list.into_iter().take(top_n).collect()
    });

    GenerationReport {
        generation,
        steps_per_gen,
        population_size,
        fitness_best: if best_fitness == f32::MIN { 0.0 } else { best_fitness },
        fitness_mean,
        fitness_median,
        fitness_std,
        fitness_iqr,
        food_eaten_total,
        food_eaten_mean,
        survival_steps_mean,
        reproductions_total: collector.reproductions_total(),
        params_mean,
        params_median,
        params_best,
        params_std,
        layers_mean,
        hidden_mean,
        mutation_rate: config.mut_rate,
        mutation_sigma: config.mut_strength,
        crossover_rate: None,
        selection_mode: config.selection_method,
        tournament_k: if matches!(config.selection_method, SelectionMethod::Tournament) {
            Some(config.tournament_k)
        } else {
            None
        },
        seed: config.seed,
        run_id: run_id.to_string(),
        config_hash: config_hash.to_string(),
        git_commit: git_commit.map(|value| value.to_string()),
        individuals,
    }
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: f32 = values.iter().sum();
    sum / values.len() as f32
}

fn mean_option(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        None
    } else {
        Some(mean(values))
    }
}

fn median(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

fn std_dev(values: &[f32], mean: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let variance = values
        .iter()
        .map(|value| {
            let diff = value - mean;
            diff * diff
        })
        .sum::<f32>()
        / values.len() as f32;
    variance.sqrt()
}

fn iqr(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let n = sorted.len();
    let q1_idx = n / 4;
    let q3_idx = (3 * n) / 4;
    sorted[q3_idx] - sorted[q1_idx]
}
