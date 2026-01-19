# EvoBrain

## Code Map

Relevant modules for the simulation loop and evolution pipeline:
- `src/simulation.rs`: main generation/step loop.
- `src/world.rs`: grid world and food queries (including FOV).
- `src/creature.rs`: creature state, actions, and tracking.
- `src/evolution.rs`: reproduction, selection, mutation, crossover.
- `src/genome.rs` / `src/neural_net.rs`: genome representation and neural net forward pass.
- `src/metrics.rs` / `src/fitness.rs`: fitness evaluation and metrics aggregation.
- `src/config.rs`: CLI/config definitions and validation.
- `src/frame_dump.rs`: optional frame snapshots.

## Simulation Features (opt-in)

Default behavior remains unchanged. New features are opt-in via CLI flags unless otherwise noted.

### Selection
- `--selection roulette|tournament` (default: `roulette`)
- `--tournament-k <u32>` (default: `5`, used with tournament selection)

### Fitness Modes
- `--fitness classic|efficient` (default: `classic`)
- Weights/tunables:
  - `--fitness-food-weight <f32>`
  - `--fitness-efficiency-weight <f32>`
  - `--fitness-survival-weight <f32>`
  - `--fitness-idle-weight <f32>`
  - `--fitness-jitter-weight <f32>`
  - `--idle-tolerance <u32>`

### Food Vision (FOV)
- `--food-vision-radius <u32>` (default: `0`, disabled)
- `--distance-metric euclidean|manhattan` (default: `euclidean`)

### Logging
- `--logging full|quick` (default: `full`)
- `--quick-keep 2|3` (default: `2`)

### Brain Architecture
- `--brain fixed|evolvable` (default: `fixed`)
- `--max-hidden-layers <u32>` (default: `4`)
- `--layer-min-neurons <u32>` (default: `4`)
- `--layer-max-neurons <u32>` (default: `64`)

### Crossover
- `--crossover none|layer|blend` (default: `none`)
- `--arch-inherit fitter|random` (default: `fitter`)
