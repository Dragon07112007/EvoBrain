use crate::genome::{genome_size, Genome};

#[derive(Debug, Clone)]
pub struct NeuralNet {
    input: usize,
    hidden: usize,
    output: usize,
    pub genome: Genome,
}

impl NeuralNet {
    pub fn new(input: usize, hidden: usize, output: usize, genome: Genome) -> Self {
        let expected = genome_size(input, hidden, output);
        assert_eq!(genome.weights.len(), expected, "Genome size mismatch");
        Self {
            input,
            hidden,
            output,
            genome,
        }
    }

    pub fn forward(&self, inputs: &[f32]) -> Vec<f32> {
        assert_eq!(inputs.len(), self.input, "Input size mismatch");
        let mut idx = 0;
        let mut hidden_vals = vec![0.0; self.hidden];
        for h in 0..self.hidden {
            let mut sum = self.genome.weights[idx];
            idx += 1;
            for i in 0..self.input {
                sum += inputs[i] * self.genome.weights[idx];
                idx += 1;
            }
            hidden_vals[h] = sum.tanh();
        }

        let mut outputs = vec![0.0; self.output];
        for o in 0..self.output {
            let mut sum = self.genome.weights[idx];
            idx += 1;
            for h in 0..self.hidden {
                sum += hidden_vals[h] * self.genome.weights[idx];
                idx += 1;
            }
            outputs[o] = sum.tanh();
        }
        outputs
    }
}
