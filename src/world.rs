use rand::Rng;

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
            if best.map_or(true, |(_, _, d)| dist_sq < d) {
                best = Some((dx, dy, dist_sq));
            }
        }
        let (dx, dy, _) = best.unwrap();
        let nx = if self.width > 1 {
            (dx / (self.width as f32 - 1.0)).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let ny = if self.height > 1 {
            (dy / (self.height as f32 - 1.0)).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        (nx, ny)
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
}
