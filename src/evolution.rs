use rand::seq::index::sample;
use rand::Rng;

use crate::config::{ArchInherit, BrainMode, Config, CrossoverMode, SelectionMethod};
use crate::creature::Creature;
use crate::fitness::compute_fitness;
use crate::genome::{genome_size_from_layers, layer_weight_ranges, Genome};
use crate::metrics::collector::MetricsCollector;
use crate::neural_net::NeuralNet;

#[derive(Debug, Clone)]
pub struct EvolutionManager {
    pub population_size: usize,
    pub elite_fraction: f32,
    pub mutation_rate: f32,
    pub mutation_strength: f32,
}

impl EvolutionManager {
    pub fn next_generation(
        &self,
        old_population: &[Creature],
        config: &Config,
        rng: &mut impl Rng,
        collector: &mut MetricsCollector,
    ) -> Vec<Creature> {
        let mut sorted = old_population.to_vec();
        sorted.sort_by(|a, b| {
            compute_fitness(b, config)
                .partial_cmp(&compute_fitness(a, config))
                .unwrap()
        });
        let elite_count = ((self.population_size as f32) * self.elite_fraction)
            .ceil()
            .max(1.0) as usize;
        let elite_count = elite_count.min(sorted.len());
        let elites = &sorted[..elite_count];
        let mut next = Vec::with_capacity(self.population_size);
        for _ in 0..self.population_size {
            let parent_a = select_parent(config, old_population, elites, rng);
            let parent_b = if matches!(config.crossover_mode, CrossoverMode::None) {
                parent_a
            } else {
                select_parent(config, old_population, elites, rng)
            };
            let mut genome = if matches!(config.crossover_mode, CrossoverMode::None) {
                parent_a.brain.genome.clone()
            } else {
                crossover_genomes(parent_a, parent_b, config, rng)
            };
            genome.mutate(self.mutation_rate, self.mutation_strength, rng);
            if matches!(config.brain_mode, BrainMode::Evolvable) {
                mutate_architecture(&mut genome, config, rng);
            }
            let brain = NeuralNet::new(genome);
            next.push(Creature::from_brain(brain, 0, 0, 0.0));
            collector.on_reproduction();
        }
        next
    }
}

pub fn random_population(size: usize, config: &Config, rng: &mut impl Rng) -> Vec<Creature> {
    let layers = config.base_layers();
    (0..size)
        .map(|_| {
            let genome = Genome::random(layers.clone(), rng);
            let brain = NeuralNet::new(genome);
            Creature::from_brain(brain, 0, 0, 0.0)
        })
        .collect()
}

fn select_parent<'a>(
    config: &Config,
    population: &'a [Creature],
    elites: &'a [Creature],
    rng: &mut impl Rng,
) -> &'a Creature {
    match config.selection_method {
        SelectionMethod::Roulette => &elites[rng.gen_range(0..elites.len())],
        SelectionMethod::Tournament => tournament_select(population, config, rng),
    }
}

fn tournament_select<'a>(
    population: &'a [Creature],
    config: &Config,
    rng: &mut impl Rng,
) -> &'a Creature {
    let k = config.tournament_k.max(2).min(population.len() as u32) as usize;
    let mut best = None;
    for idx in sample(rng, population.len(), k).iter() {
        let candidate = &population[idx];
        let fitness = compute_fitness(candidate, config);
        let replace = match best {
            Some((_, best_fit)) => fitness > best_fit,
            None => true,
        };
        if replace {
            best = Some((candidate, fitness));
        }
    }
    best.expect("population must be non-empty").0
}

fn crossover_genomes(
    parent_a: &Creature,
    parent_b: &Creature,
    config: &Config,
    rng: &mut impl Rng,
) -> Genome {
    let (arch_parent, other_parent) = match config.brain_mode {
        BrainMode::Fixed => (parent_a, parent_b),
        BrainMode::Evolvable => match config.arch_inherit {
            ArchInherit::Random => {
                if rng.gen::<bool>() {
                    (parent_a, parent_b)
                } else {
                    (parent_b, parent_a)
                }
            }
            ArchInherit::Fitter => {
                if compute_fitness(parent_a, config) >= compute_fitness(parent_b, config) {
                    (parent_a, parent_b)
                } else {
                    (parent_b, parent_a)
                }
            }
        },
    };
    let layers = arch_parent.brain.genome.layers.clone();
    let mut genome = Genome {
        layers: layers.clone(),
        weights: vec![0.0; genome_size_from_layers(&layers)],
    };
    let child_ranges = layer_weight_ranges(&layers);
    let a_ranges = layer_weight_ranges(&arch_parent.brain.genome.layers);
    let b_ranges = layer_weight_ranges(&other_parent.brain.genome.layers);
    for (idx, child_range) in child_ranges.iter().enumerate() {
        let layer_match_a = same_layer_shape(&layers, &arch_parent.brain.genome.layers, idx);
        let layer_match_b = same_layer_shape(&layers, &other_parent.brain.genome.layers, idx);
        let both_match = layer_match_a && layer_match_b;
        if matches!(config.crossover_mode, CrossoverMode::Blend) && both_match {
            let alpha = rng.gen::<f32>();
            let a_slice = &arch_parent.brain.genome.weights[a_ranges[idx].clone()];
            let b_slice = &other_parent.brain.genome.weights[b_ranges[idx].clone()];
            for (offset, weight) in genome.weights[child_range.clone()].iter_mut().enumerate() {
                *weight = alpha * a_slice[offset] + (1.0 - alpha) * b_slice[offset];
            }
            continue;
        }
        let source = match (layer_match_a, layer_match_b) {
            (true, true) => {
                if matches!(config.crossover_mode, CrossoverMode::Layer) && rng.gen::<bool>() {
                    LayerSource::Other
                } else {
                    LayerSource::Arch
                }
            }
            (true, false) => LayerSource::Arch,
            (false, true) => LayerSource::Other,
            (false, false) => LayerSource::Random,
        };
        match source {
            LayerSource::Arch => {
                let src = &arch_parent.brain.genome.weights[a_ranges[idx].clone()];
                genome.weights[child_range.clone()].copy_from_slice(src);
            }
            LayerSource::Other => {
                let src = &other_parent.brain.genome.weights[b_ranges[idx].clone()];
                genome.weights[child_range.clone()].copy_from_slice(src);
            }
            LayerSource::Random => {
                for weight in &mut genome.weights[child_range.clone()] {
                    *weight = rng.gen_range(-1.0..=1.0);
                }
            }
        }
    }
    genome
}

fn same_layer_shape(child_layers: &[usize], parent_layers: &[usize], idx: usize) -> bool {
    if idx + 1 >= child_layers.len() || idx + 1 >= parent_layers.len() {
        return false;
    }
    child_layers[idx] == parent_layers[idx] && child_layers[idx + 1] == parent_layers[idx + 1]
}

fn mutate_architecture(genome: &mut Genome, config: &Config, rng: &mut impl Rng) {
    if rng.gen::<f32>() >= config.mut_rate {
        return;
    }
    let hidden_layers = genome.layers.len().saturating_sub(2);
    let can_add = hidden_layers < config.max_hidden_layers as usize;
    let can_remove = hidden_layers > 1;
    let mut ops = Vec::new();
    if can_add {
        ops.push(ArchMutation::Add);
    }
    if can_remove {
        ops.push(ArchMutation::Remove);
    }
    ops.push(ArchMutation::Resize);
    let choice = ops[rng.gen_range(0..ops.len())];
    let mut layers = genome.layers.clone();
    match choice {
        ArchMutation::Add => {
            let insert_idx = rng.gen_range(1..layers.len() - 1);
            let size = rng.gen_range(config.layer_min_neurons..=config.layer_max_neurons) as usize;
            layers.insert(insert_idx, size);
        }
        ArchMutation::Remove => {
            let remove_idx = rng.gen_range(1..layers.len() - 1);
            layers.remove(remove_idx);
        }
        ArchMutation::Resize => {
            let layer_idx = rng.gen_range(1..layers.len() - 1);
            let delta = rng.gen_range(-4..=4);
            let current = layers[layer_idx] as i32;
            let new_size = (current + delta).clamp(
                config.layer_min_neurons as i32,
                config.layer_max_neurons as i32,
            ) as usize;
            layers[layer_idx] = new_size.max(config.layer_min_neurons as usize);
        }
    }
    if layers != genome.layers {
        genome.reset_for_layers(layers, rng);
    }
}

#[derive(Clone, Copy)]
enum LayerSource {
    Arch,
    Other,
    Random,
}

#[derive(Clone, Copy)]
enum ArchMutation {
    Add,
    Remove,
    Resize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn tournament_select_picks_best_in_sample() {
        let mut rng = StdRng::seed_from_u64(1);
        let config = Config {
            generations: 1,
            population: 3,
            width: 1,
            height: 1,
            food: 0,
            max_steps: 1,
            max_energy: 10.0,
            move_cost: 1.0,
            food_energy: 1.0,
            seed: 1,
            input: 4,
            hidden: 2,
            output: 4,
            selection_method: SelectionMethod::Tournament,
            tournament_k: 3,
            elite: 0.2,
            mut_rate: 0.1,
            mut_strength: 0.1,
            fitness_mode: crate::config::FitnessMode::Classic,
            fitness_food_weight: 1.0,
            fitness_efficiency_weight: 1.0,
            fitness_survival_weight: 0.1,
            fitness_idle_weight: 0.5,
            fitness_jitter_weight: 0.2,
            idle_tolerance: 10,
            logging_mode: crate::config::LoggingMode::Full,
            log_gens: "all".to_string(),
            full_log_gens: None,
            run_id: None,
            quick_keep: 2,
            food_vision_radius: 0,
            distance_metric: crate::config::DistanceMetric::Euclidean,
            brain_mode: BrainMode::Fixed,
            max_hidden_layers: 4,
            layer_min_neurons: 4,
            layer_max_neurons: 64,
            crossover_mode: CrossoverMode::None,
            arch_inherit: ArchInherit::Fitter,
            out: "unused.csv".to_string(),
            run_metadata: "unused.json".to_string(),
            dump_frames: false,
            frame_every: 1,
            frames_dir: "frames".to_string(),
            progress: 0,
        };
        let layers = config.base_layers();
        let mut pop = vec![
            Creature::from_brain(
                NeuralNet::new(Genome::random(layers.clone(), &mut rng)),
                0,
                0,
                0.0,
            ),
            Creature::from_brain(
                NeuralNet::new(Genome::random(layers.clone(), &mut rng)),
                0,
                0,
                0.0,
            ),
            Creature::from_brain(
                NeuralNet::new(Genome::random(layers.clone(), &mut rng)),
                0,
                0,
                0.0,
            ),
        ];
        pop[0].age = 1;
        pop[1].age = 5;
        pop[2].age = 3;
        let selected = tournament_select(&pop, &config, &mut rng);
        assert_eq!(selected.age, 5);
    }

    #[test]
    fn crossover_handles_shape_mismatch() {
        let mut rng = StdRng::seed_from_u64(2);
        let mut config = Config {
            generations: 1,
            population: 2,
            width: 1,
            height: 1,
            food: 0,
            max_steps: 1,
            max_energy: 10.0,
            move_cost: 1.0,
            food_energy: 1.0,
            seed: 1,
            input: 4,
            hidden: 2,
            output: 4,
            selection_method: SelectionMethod::Roulette,
            tournament_k: 3,
            elite: 0.2,
            mut_rate: 0.1,
            mut_strength: 0.1,
            fitness_mode: crate::config::FitnessMode::Classic,
            fitness_food_weight: 1.0,
            fitness_efficiency_weight: 1.0,
            fitness_survival_weight: 0.1,
            fitness_idle_weight: 0.5,
            fitness_jitter_weight: 0.2,
            idle_tolerance: 10,
            logging_mode: crate::config::LoggingMode::Full,
            log_gens: "all".to_string(),
            full_log_gens: None,
            run_id: None,
            quick_keep: 2,
            food_vision_radius: 0,
            distance_metric: crate::config::DistanceMetric::Euclidean,
            brain_mode: BrainMode::Evolvable,
            max_hidden_layers: 4,
            layer_min_neurons: 2,
            layer_max_neurons: 8,
            crossover_mode: CrossoverMode::Blend,
            arch_inherit: ArchInherit::Random,
            out: "unused.csv".to_string(),
            run_metadata: "unused.json".to_string(),
            dump_frames: false,
            frame_every: 1,
            frames_dir: "frames".to_string(),
            progress: 0,
        };
        let genome_a = Genome::random(vec![4, 2, 4], &mut rng);
        let genome_b = Genome::random(vec![4, 3, 3, 4], &mut rng);
        let parent_a = Creature::from_brain(NeuralNet::new(genome_a), 0, 0, 0.0);
        let parent_b = Creature::from_brain(NeuralNet::new(genome_b), 0, 0, 0.0);
        let child = crossover_genomes(&parent_a, &parent_b, &config, &mut rng);
        assert!(
            child.layers == parent_a.brain.genome.layers
                || child.layers == parent_b.brain.genome.layers
        );
        assert_eq!(child.weights.len(), genome_size_from_layers(&child.layers));
        config.arch_inherit = ArchInherit::Fitter;
        let child_b = crossover_genomes(&parent_a, &parent_b, &config, &mut rng);
        assert_eq!(
            child_b.weights.len(),
            genome_size_from_layers(&child_b.layers)
        );
    }
}
