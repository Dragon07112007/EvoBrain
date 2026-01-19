use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Genome {
    pub weights: Vec<f32>,
}

impl Genome {
    pub fn random(size: usize, rng: &mut impl Rng) -> Self {
        let weights = (0..size)
            .map(|_| rng.gen_range(-1.0..=1.0))
            .collect();
        Self { weights }
    }

    pub fn mutate(&mut self, rate: f32, strength: f32, rng: &mut impl Rng) {
        let mut mutated = false;
        for weight in &mut self.weights {
            if rng.gen::<f32>() < rate {
                let delta = rng.gen_range(-strength..=strength);
                *weight += delta;
                mutated = true;
            }
        }
        if !mutated && rate >= 1.0 && strength > 0.0 && !self.weights.is_empty() {
            let idx = rng.gen_range(0..self.weights.len());
            let delta = if rng.gen::<bool>() { strength } else { -strength };
            self.weights[idx] += delta;
        }
    }
}

pub fn genome_size(input: usize, hidden: usize, output: usize) -> usize {
    (input + 1) * hidden + (hidden + 1) * output
}
