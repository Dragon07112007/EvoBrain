use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Genome {
    pub layers: Vec<usize>,
    pub weights: Vec<f32>,
}

impl Genome {
    pub fn random(layers: Vec<usize>, rng: &mut impl Rng) -> Self {
        let size = genome_size_from_layers(&layers);
        let weights = (0..size).map(|_| rng.gen_range(-1.0..=1.0)).collect();
        Self { layers, weights }
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
            let delta = if rng.gen::<bool>() {
                strength
            } else {
                -strength
            };
            self.weights[idx] += delta;
        }
    }

    pub fn reset_for_layers(&mut self, layers: Vec<usize>, rng: &mut impl Rng) {
        let size = genome_size_from_layers(&layers);
        self.layers = layers;
        self.weights = (0..size).map(|_| rng.gen_range(-1.0..=1.0)).collect();
    }
}

pub fn genome_size(input: usize, hidden: usize, output: usize) -> usize {
    genome_size_from_layers(&[input, hidden, output])
}

pub fn genome_size_from_layers(layers: &[usize]) -> usize {
    if layers.len() < 2 {
        return 0;
    }
    layers.windows(2).map(|pair| (pair[0] + 1) * pair[1]).sum()
}

pub fn layer_weight_ranges(layers: &[usize]) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::with_capacity(layers.len().saturating_sub(1));
    let mut start = 0;
    for pair in layers.windows(2) {
        let len = (pair[0] + 1) * pair[1];
        let end = start + len;
        ranges.push(start..end);
        start = end;
    }
    ranges
}
