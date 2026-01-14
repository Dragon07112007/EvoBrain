use serde::Serialize;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::creature::Creature;
use crate::world::World;

#[derive(Debug, Serialize)]
pub struct FrameCreature {
    pub x: usize,
    pub y: usize,
    pub energy: f32,
    pub alive: bool,
}

#[derive(Debug, Serialize)]
pub struct FrameDump {
    pub generation: usize,
    pub step: usize,
    pub width: usize,
    pub height: usize,
    pub food: Vec<(usize, usize)>,
    pub creatures: Vec<FrameCreature>,
}

pub fn dump_frame(
    dir: &str,
    generation: usize,
    step: usize,
    world: &World,
    creatures: &[Creature],
) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let frame = FrameDump {
        generation,
        step,
        width: world.width,
        height: world.height,
        food: world.food.clone(),
        creatures: creatures
            .iter()
            .map(|c| FrameCreature {
                x: c.x,
                y: c.y,
                energy: c.energy,
                alive: c.alive,
            })
            .collect(),
    };
    let path = Path::new(dir).join(format!("gen{:04}_step{:04}.json", generation, step));
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &frame)?;
    writer.write_all(b"\n")?;
    Ok(())
}
