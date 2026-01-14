use rand::Rng;

use crate::creature::Creature;
use crate::genome::{genome_size, Genome};
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
        nn_sizes: (usize, usize, usize),
        rng: &mut impl Rng,
    ) -> Vec<Creature> {
        let mut sorted = old_population.to_vec();
        sorted.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap());
        let elite_count = ((self.population_size as f32) * self.elite_fraction)
            .ceil()
            .max(1.0) as usize;
        let elite_count = elite_count.min(sorted.len());
        let elites = &sorted[..elite_count];
        let mut next = Vec::with_capacity(self.population_size);
        for _ in 0..self.population_size {
            let parent = &elites[rng.gen_range(0..elites.len())];
            let mut genome = parent.brain.genome.clone();
            genome.mutate(self.mutation_rate, self.mutation_strength, rng);
            let (input, hidden, output) = nn_sizes;
            let brain = NeuralNet::new(input, hidden, output, genome);
            next.push(Creature::from_brain(brain, 0, 0, 0.0));
        }
        next
    }
}

pub fn random_population(
    size: usize,
    nn_sizes: (usize, usize, usize),
    rng: &mut impl Rng,
) -> Vec<Creature> {
    let (input, hidden, output) = nn_sizes;
    let genome_len = genome_size(input, hidden, output);
    (0..size)
        .map(|_| {
            let genome = Genome::random(genome_len, rng);
            let brain = NeuralNet::new(input, hidden, output, genome);
            Creature::from_brain(brain, 0, 0, 0.0)
        })
        .collect()
}
