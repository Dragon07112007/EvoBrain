use rand::Rng;

use crate::config::DistanceMetric;

#[derive(Debug, Clone)]
pub struct World {
    pub width: usize,
    pub height: usize,
    pub food: Vec<(usize, usize)>,
}

impl World {
    pub fn new(width: usize, height: usize, food_count: usize, rng: &mut impl Rng) -> Self {
        let mut food = Vec::with_capacity(food_count);
        for _ in 0..food_count {
            food.push(Self::random_pos(width, height, rng));
        }
        Self {
            width,
            height,
            food,
        }
    }

    pub fn nearest_food(&self, x: usize, y: usize) -> (f32, f32) {
        if self.food.is_empty() {
            return (0.0, 0.0);
        }
        let mut best = None;
        for &(fx, fy) in &self.food {
            let dx = fx as f32 - x as f32;
            let dy = fy as f32 - y as f32;
            let dist_sq = dx * dx + dy * dy;
            let replace = match best {
                Some((_, _, d)) => dist_sq < d,
                None => true,
            };
            if replace {
                best = Some((dx, dy, dist_sq));
            }
        }
        let (dx, dy, _) = best.unwrap();
        Self::normalize_vector(dx, dy, self.width, self.height)
    }

    pub fn nearest_food_within(
        &self,
        x: usize,
        y: usize,
        radius: u32,
        metric: DistanceMetric,
    ) -> Option<(f32, f32)> {
        if self.food.is_empty() {
            return None;
        }
        let radius = radius as i32;
        let mut best = None;
        for &(fx, fy) in &self.food {
            let dx = fx as i32 - x as i32;
            let dy = fy as i32 - y as i32;
            let dist = match metric {
                DistanceMetric::Euclidean => (dx * dx + dy * dy) as f32,
                DistanceMetric::Manhattan => (dx.abs() + dy.abs()) as f32,
            };
            let within = match metric {
                DistanceMetric::Euclidean => dist <= (radius * radius) as f32,
                DistanceMetric::Manhattan => dist <= radius as f32,
            };
            if within {
                let replace = match best {
                    Some((_, _, best_dist)) => dist < best_dist,
                    None => true,
                };
                if replace {
                    best = Some((dx as f32, dy as f32, dist));
                }
            }
        }
        best.map(|(dx, dy, _)| Self::normalize_vector(dx, dy, self.width, self.height))
    }

    pub fn try_eat_food(&mut self, x: usize, y: usize, rng: &mut impl Rng) -> bool {
        if let Some(idx) = self.food.iter().position(|&(fx, fy)| fx == x && fy == y) {
            self.food[idx] = Self::random_pos(self.width, self.height, rng);
            return true;
        }
        false
    }

    fn random_pos(width: usize, height: usize, rng: &mut impl Rng) -> (usize, usize) {
        let x = rng.gen_range(0..width.max(1));
        let y = rng.gen_range(0..height.max(1));
        (x, y)
    }

    fn normalize_vector(dx: f32, dy: f32, width: usize, height: usize) -> (f32, f32) {
        let nx = if width > 1 {
            (dx / (width as f32 - 1.0)).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let ny = if height > 1 {
            (dy / (height as f32 - 1.0)).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        (nx, ny)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fov_blocks_food_outside_radius() {
        let world = World {
            width: 10,
            height: 10,
            food: vec![(4, 0)],
        };
        let seen = world.nearest_food_within(0, 0, 3, DistanceMetric::Euclidean);
        assert!(seen.is_none());
        let seen = world.nearest_food_within(0, 0, 4, DistanceMetric::Euclidean);
        assert!(seen.is_some());
    }
}
