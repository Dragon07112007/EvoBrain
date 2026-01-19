use crate::genome::{genome_size_from_layers, Genome};

#[derive(Debug, Clone)]
pub struct NeuralNet {
    pub genome: Genome,
}

impl NeuralNet {
    pub fn new(genome: Genome) -> Self {
        let expected = genome_size_from_layers(&genome.layers);
        assert_eq!(genome.weights.len(), expected, "Genome size mismatch");
        Self { genome }
    }

    pub fn forward(&self, inputs: &[f32]) -> Vec<f32> {
        let layers = &self.genome.layers;
        let input_size = layers.first().copied().unwrap_or(0);
        let output_size = layers.last().copied().unwrap_or(0);
        assert_eq!(inputs.len(), input_size, "Input size mismatch");
        let mut idx = 0;
        let mut prev_vals = inputs.to_vec();
        for pair in layers.windows(2) {
            let next_size = pair[1];
            let mut next_vals = vec![0.0; next_size];
            for next_val in &mut next_vals {
                let mut sum = self.genome.weights[idx];
                idx += 1;
                for &val in &prev_vals {
                    sum += val * self.genome.weights[idx];
                    idx += 1;
                }
                *next_val = sum.tanh();
            }
            prev_vals = next_vals;
        }
        assert_eq!(prev_vals.len(), output_size, "Output size mismatch");
        prev_vals
    }
}
