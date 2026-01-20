use rand::Rng;

use crate::neural_net::NeuralNet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub x: usize,
    pub y: usize,
    pub energy: f32,
    pub age: u32,
    pub alive: bool,
    pub brain: NeuralNet,
    pub food_collected: u32,
    pub energy_spent: f32,
    pub survival_steps: u32,
    pub idle_steps: u32,
    pub jitter_score: u32,
    last_action: Option<Action>,
}

impl Creature {
    pub fn from_brain(brain: NeuralNet, x: usize, y: usize, energy: f32) -> Self {
        Self {
            x,
            y,
            energy,
            age: 0,
            alive: true,
            brain,
            food_collected: 0,
            energy_spent: 0.0,
            survival_steps: 0,
            idle_steps: 0,
            jitter_score: 0,
            last_action: None,
        }
    }

    pub fn perceive(&self, dx: f32, dy: f32, max_energy: f32, rng: &mut impl Rng) -> Vec<f32> {
        let energy_norm = if max_energy > 0.0 {
            (self.energy / max_energy).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let noise = rng.gen_range(-1.0..=1.0);
        vec![dx, dy, energy_norm, noise]
    }

    pub fn decide(&self, inputs: &[f32]) -> Action {
        let outputs = self.brain.forward(inputs);
        let mut best_idx = 0;
        let mut best_val = outputs[0];
        for (idx, &val) in outputs.iter().enumerate().skip(1) {
            if val > best_val {
                best_val = val;
                best_idx = idx;
            }
        }
        match best_idx {
            0 => Action::Up,
            1 => Action::Down,
            2 => Action::Left,
            _ => Action::Right,
        }
    }

    pub fn act(&mut self, action: Action, world_w: usize, world_h: usize, move_cost: f32) {
        if !self.alive {
            return;
        }
        let start_x = self.x;
        let start_y = self.y;
        if let Some(last) = self.last_action {
            if last != action {
                self.jitter_score = self.jitter_score.saturating_add(1);
            }
        }
        self.last_action = Some(action);
        match action {
            Action::Up => {
                if self.y > 0 {
                    self.y -= 1;
                }
            }
            Action::Down => {
                if self.y + 1 < world_h {
                    self.y += 1;
                }
            }
            Action::Left => {
                if self.x > 0 {
                    self.x -= 1;
                }
            }
            Action::Right => {
                if self.x + 1 < world_w {
                    self.x += 1;
                }
            }
        }
        self.energy -= move_cost;
        self.energy_spent += move_cost;
        self.age += 1;
        self.survival_steps += 1;
        if self.x == start_x && self.y == start_y {
            self.idle_steps = self.idle_steps.saturating_add(1);
        }
        if self.energy <= 0.0 {
            self.alive = false;
        }
    }

    pub fn fitness_classic(&self) -> f32 {
        self.age as f32 + self.energy.max(0.0)
    }

    pub fn reset_tracking(&mut self) {
        self.food_collected = 0;
        self.energy_spent = 0.0;
        self.survival_steps = 0;
        self.idle_steps = 0;
        self.jitter_score = 0;
        self.last_action = None;
    }
}
