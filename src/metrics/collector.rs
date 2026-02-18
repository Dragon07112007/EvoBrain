#[derive(Debug, Default, Clone)]
pub struct MetricsCollector {
    food_eaten_total: u32,
    reproductions_total: u32,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_food_eaten(&mut self, amount: u32) {
        self.food_eaten_total = self.food_eaten_total.saturating_add(amount);
    }

    pub fn on_reproduction(&mut self) {
        self.reproductions_total = self.reproductions_total.saturating_add(1);
    }

    pub fn food_eaten_total(&self) -> u32 {
        self.food_eaten_total
    }

    pub fn reproductions_total(&self) -> u32 {
        self.reproductions_total
    }

    pub fn reset(&mut self) {
        self.food_eaten_total = 0;
        self.reproductions_total = 0;
    }
}
