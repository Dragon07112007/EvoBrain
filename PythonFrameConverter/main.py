
import json
import os
import re
import matplotlib.pyplot as plt

# ==========================
# Configuration
# ==========================
DATA_DIR = "data"
OUTPUT_DIR = "output/frames"
SNAPSHOT_STEP_INTERVAL = 1  # save every N steps
DPI = 120

# Matches: gen0000_step0000.json
FILENAME_RE = re.compile(r"^gen(\d{4})_step(\d{4})\.json$")

os.makedirs(OUTPUT_DIR, exist_ok=True)


def parse_gen_step(filename: str):
    m = FILENAME_RE.match(filename)
    if not m:
        return None
    gen = int(m.group(1))
    step = int(m.group(2))
    return gen, step


# ==========================
# Visualization
# ==========================
def render_snapshot(snapshot: dict, frame_index: int):
    width = snapshot["width"]
    height = snapshot["height"]
    generation = snapshot["generation"]
    step = snapshot["step"]

    food = snapshot.get("food", [])
    creatures = snapshot.get("creatures", [])

    fig, ax = plt.subplots(figsize=(6, 6))

    # Food
    if food:
        fx, fy = zip(*food)
        ax.scatter(fx, fy, c="green", s=15, label="Food", alpha=0.7)

    # Creatures
    alive_x, alive_y = [], []
    dead_x, dead_y = [], []

    for c in creatures:
        if c.get("alive", True):
            alive_x.append(c["x"])
            alive_y.append(c["y"])
        else:
            dead_x.append(c["x"])
            dead_y.append(c["y"])

    if alive_x:
        ax.scatter(alive_x, alive_y, c="blue", s=20, label="Alive", alpha=0.9)
    if dead_x:
        ax.scatter(dead_x, dead_y, c="red", s=20, label="Dead", alpha=0.9)

    # Formatting
    ax.set_xlim(-0.5, width - 0.5)
    ax.set_ylim(-0.5, height - 0.5)
    ax.set_xticks([])
    ax.set_yticks([])
    ax.set_aspect("equal")
    ax.set_title(f"Generation {generation} | Step {step}")
    ax.legend(loc="upper right", fontsize=8)

    # ✅ NEW naming: snapshot index, not raw step
    out_name = f"gen{generation:04d}_step{frame_index:04d}.png"
    out_path = os.path.join(OUTPUT_DIR, out_name)

    plt.savefig(out_path, dpi=DPI, bbox_inches="tight")
    plt.close(fig)


# ==========================
# Main
# ==========================
def main():
    entries = []

    for fn in os.listdir(DATA_DIR):
        parsed = parse_gen_step(fn)
        if parsed is not None:
            gen, step = parsed
            entries.append((gen, step, fn))

    # Sort by generation then simulation step
    entries.sort(key=lambda t: (t[0], t[1]))

    if not entries:
        print("No matching JSON files found.")
        return

    for gen, step, fn in entries:
        if step % SNAPSHOT_STEP_INTERVAL != 0:
            continue

        frame_index = step // SNAPSHOT_STEP_INTERVAL

        with open(os.path.join(DATA_DIR, fn), "r", encoding="utf-8") as f:
            snapshot = json.load(f)

        render_snapshot(snapshot, frame_index)
        print(f"Rendered gen{gen:04d}_step{frame_index:03d}.png")


if __name__ == "__main__":
    main()
