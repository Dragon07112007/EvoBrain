Diese Prompts hier sind die Prompts, die als Anweisung für Codex genommen wurde. Das heißt sie wurden exakt so in  Codex rein koopiert um die EvoBrain Simulation zu erstellen. Diese 3. Prompts sind aus Konversation generiert worden, die mit ChatGPT geführt worden sind. In diesen Konversation habe ich das Ziel und ide Art und Weise geschildert was die Simulation alles können muss.

1. Prompt:


Du bist Codex und arbeitest in einem leeren Ordner. Erstelle ein vollständiges Rust-Projekt (cargo) für eine headless Evolutionssimulation mit neuronalen Netzen auf einem 2D-Grid. Fokus: reproduzierbare Experimente + CSV-Datenexport. Keine Live-View. Optional: Frame-Dumps, aus denen später ein Video gebaut werden kann (PNGs optional; notfalls ASCII/JSON pro Step).

WICHTIG:
- Liefere Code-Änderungen als echte Dateien im Repo.
- Sorge dafür, dass `cargo test` und `cargo run --release -- ...` funktionieren.
- Nutze klare, wartbare Struktur, Kommentare wo sinnvoll, und sinnvolle Defaults.
- Keine externen ML-Frameworks. Keine Backpropagation. Evolution über Mutation/Selektion.
- Reproduzierbarkeit: alle Zufallsprozesse laufen über seeded RNG.
- Implementiere robuste Fehlerbehandlung für CLI-Parsing und File-I/O.
- Halte die Simulation performant genug für z.B. 200 Population, 100 Generationen.

TECH-STACK (Rust):
- Edition 2021
- Dependencies:
  - rand = "0.8"
  - clap = { version = "4", features = ["derive"] }
  - csv = "1"
  - serde = { version = "1", features = ["derive"] }
  - serde_json = "1"
(Keine Grafik-Dependency nötig.)

PROJEKTSTRUKTUR:
src/
  main.rs
  lib.rs
  simulation.rs
  config.rs
  world.rs
  creature.rs
  neural_net.rs
  genome.rs
  evolution.rs
  metrics.rs
  frame_dump.rs   (optional, aber implementieren)
tests/
  smoke.rs

KONZEPT:
1) World:
- diskretes Grid: width, height
- Nahrung als Vec<(x,y)> oder HashSet; entscheide pragmatisch (Performance ok).
- `nearest_food(x,y)` -> normierten Richtungsvektor (dx,dy) (float in [-1,1]).
- `try_eat_food(x,y)` -> bool; wenn gegessen: respawn Nahrung an zufälliger Position (seeded RNG).
- RNG: StdRng seeded. Seed pro Generation oder global, aber deterministisch.

2) Creature:
- Felder: x, y, energy, age, alive, brain (NeuralNet)
- Methoden:
  - perceive(dx,dy,max_energy,rng)-> Vec<f32> (z.B. 4 inputs: dx,dy, energy/max_energy, noise in [-1,1])
  - decide(inputs)-> Action (Up/Down/Left/Right) via argmax der Output-Neuronen
  - act(action, world_w, world_h, move_cost) -> verändert pos, energy, age, alive
  - fitness() -> f32 (Default: age + max(0,energy); später anpassbar)

3) NeuralNet:
- Feedforward: Input -> Hidden -> Output
- Activation: tanh
- Genom = alle Gewichte inkl. Bias:
  - (input+1)*hidden + (hidden+1)*output
- forward(&self, inputs)-> Vec<f32>
- NeuralNet besitzt Genome (oder hält ref); implementiere sauber.

4) Genome:
- Vec<f32> weights
- random(size, rng) -> Genome
- mutate(rate, strength, rng): pro weight mit Wahrscheinlichkeit `rate` additive Mutation in [-strength, strength]

5) EvolutionManager:
- Parameter: population_size, elite_fraction, mutation_rate, mutation_strength
- next_generation(old_pop, nn_sizes, rng) -> Vec<Creature>
- Selektion: Elitismus (top elite_fraction) + Rest durch sampling aus Elite (oder Turnier; entscheide und dokumentiere)
- Reproduktion: clone parent genome -> mutate -> neuer NeuralNet -> neues Creature (Position/Energie werden in Simulation gesetzt)

6) Simulation Loop:
- Diskrete Generationen.
- Pro Generation:
  - World initialisieren (oder persistieren, aber deterministisch!)
  - Population initialisieren (Gen0 random, spätere aus EvolutionManager) und in zufälligen Startpositionen verteilen (seeded RNG).
  - Steps laufen bis max_steps oder alle tot:
    - Für jede lebende Kreatur: nearest_food -> perceive -> decide -> act
    - Wenn try_eat_food true: energy += food_energy (clamp optional auf max_energy)
  - Metrics berechnen und speichern.

7) Metrics + CSV Export:
- Pro Generation: generation, avg_fitness, max_fitness, avg_age, survivors, avg_energy (optional), food_eaten_total (optional)
- Schreibe CSV (results.csv) über csv crate.
- Zusätzlich: schreibe run metadata als JSON (config, seeds, nn sizes) in z.B. run.json, damit Experimente nachvollziehbar sind.

8) Frame Dump (optional, aber implementieren):
- CLI-Flag `--dump-frames` und `--frame-every N` und `--frames-dir frames/`
- Wenn aktiviert: pro N steps einer Generation speichere eine einfache JSON-Datei:
  - { generation, step, width,height, food:[...], creatures:[{x,y,energy,alive}] }
- Kein PNG nötig. Nur Daten für späteres Rendering/Video.

CLI (clap):
- `--generations` (default 100)
- `--population` (default 200)
- `--width` `--height` (default 50/50)
- `--food` (default 100)
- `--max-steps` (default 1000)
- `--max-energy` (default 100)
- `--move-cost` (default 1)
- `--food-energy` (default 30)
- `--seed` (default 42)
- NN sizes: `--input` default 4, `--hidden` default 8, `--output` default 4
- Evolution params: `--elite` default 0.1, `--mut-rate` default 0.05, `--mut-strength` default 0.2
- Output: `--out results.csv` default
- Optional frame dump flags wie oben
- Optional: `--progress` prints kurze status line alle X generationen (ohne TUI)

AKZEPTANZKRITERIEN:
- `cargo test` läuft durch.
- `cargo run --release -- --generations 10 --population 50 --seed 123` erzeugt results.csv und run.json.
- CSV hat Header und 10 Zeilen (für Generation 0..9).
- Mit gleichem Seed + gleichen Parametern sind outputs deterministisch (zumindest metrics identisch).
- Code ist modular, gut benannt, und ohne ungenutzte warnings (`cargo clippy` wäre nice-to-have, aber nicht zwingend).

TESTS:
- smoke test: run 3 generationen mit kleiner config und prüfe:
  - CSV wird geschrieben und hat erwartete Anzahl Zeilen
  - Determinismus: zwei Runs mit gleichem Seed ergeben gleiche Metrics (in-memory Vergleich, ohne Dateisystem wenn möglich)
- Unit Test für genome_size und Mutation (Mutation verändert bei rate=1.0 garantiert etwas, mit strength>0)

IMPLEMENTIERUNGSDETAILS:
- Nutze einen zentralen `StdRng` (seeded) und gib ihn als &mut an Funktionen weiter; kein thread_rng in Simulationscode.
- Vermeide float NaN issues bei partial_cmp: beim argmax der Outputs verwende eine sichere Vergleichsfunktion (z.B. unwrap nur wenn garantiert keine NaNs entstehen; tanh macht keine NaNs; inputs begrenzen).
- Performance: food lookup kann linear bleiben bei food_count ~100; ok. Wenn du willst, nutze HashSet für try_eat_food.
- Dokumentiere in Kommentaren kurz, dass Evolution statt Training verwendet wird.

LIEFERUMFANG:
- Erstelle das komplette Projekt mit allen Dateien, implementiere alles wie spezifiziert.
- Gib am Ende kurze Anleitung (nur wenige Zeilen) wie man es ausführt und welche Files entstehen.



2. Prompt:

DU BIST CODEX (Senior Rust Engineer). Arbeite direkt im bestehenden Repository („EvoBrain Simulation“) und baue auf dem aktuellen Stand auf. Ziel: Neue Evolutions-/Simulations-Features implementieren, ohne die Grundsimulation im Default-Verhalten zu verändern.

WICHTIGSTE REGELN
- NICHT die bestehende Config/CLI-Parsing-Architektur neu erfinden. Es existiert bereits ein Cargo/clap (oder ähnlicher) Config-Parser: ERWEITERN statt ersetzen.
- Default-Run ohne neue Flags muss sich „im Groben“ wie bisher verhalten (gleiche Logikpfade, gleiche Sensorik, gleiche Selektion/Fitness, gleiche Logs – sofern nicht reine Zusatz-Stats).
- Alle neuen Features sind per Cargo CLI auswählbar (opt-in), außer absolut harmlose/strukturelle Verbesserungen (z. B. zusätzliche Stats-Datei ohne Verhaltensänderung).
- Determinismus: Wenn es bereits einen Seed gibt, muss das Verhalten bei gleichem Seed + gleichen Flags reproduzierbar sein.
- Performance: FOV-Radius-Scan effizient halten.

DEIN VORGEHEN (ohne Rückfragen)
1) Repo-Scan
   - Finde: Simulation-Loop, Kreaturen/DNA/Brain-Strukturen, Reproduktion, Mutation, Fitness-Berechnung, Selektion, Food-Detection/Sensorik, Logging/Snapshot-System, bestehende Config/CLI-Args.
   - Notiere kurz in Kommentaren/README welche Dateien/Module relevant sind.

2) Bestehende Config erweitern (kein Rewrite)
   - Füge neue Felder zur bestehenden Config-Struktur hinzu.
   - Registriere neue CLI-Flags in der bestehenden clap/arg-Definition.
   - Stelle sicher: Defaults = „Feature aus“ und damit alter Codepfad.

NEUE FEATURES (Implementierungsvorgaben)

A) FOV (Food-Detection im Umkreis von Cells)
Ziel: Wenn FOV aktiv, darf eine Kreatur Nahrung nur erkennen, wenn sie innerhalb eines Radius R (in Zellkoordinaten) liegt.
- KEIN Sichtkegel, KEINE Rays, KEINE Winkel – nur Umkreis.
- Neue Config:
  - food_vision_radius: u32 (0 = aus / default 0)
  - distance_metric: (optional) enum {euclidean, manhattan} – default euclidean; wenn optional weglassen, nimm euclidean fest.
- Umsetzung:
  - Bei Sensorik/Food-Query: Suche die nächste Food-Zelle innerhalb dist <= R.
  - Distanz: euclidean (sqrt nicht nötig; vergleiche squared distance).
  - Wenn keine Food in Range: Sensor liefert „kein Ziel“ (z. B. dx=0, dy=0, found=0).
  - Achte darauf, dass Input-Dimension des Brain stabil bleibt. Falls bisher „global nearest food vector“ existiert: ersetze NICHT die Dimension, sondern setze Werte bei FOV-Aus wie bisher; bei FOV-An setze „nichts gesehen“ = 0.

B) Turnierselektion (Option)
Ziel: Alternative Eltern-Auswahl (ohne Default zu ändern).
- Neue Config:
  - selection_method: enum {roulette, tournament} (default roulette / aktuelles Verhalten)
  - tournament_k: u32 (nur genutzt wenn tournament; default 5)
  - optional tournament_p: f32 (falls bereits Mechanik für probabilistische Auswahl existiert; ansonsten weglassen)
- Umsetzung:
  - Implementiere einen sauberen Selektionspfad im Reproduktionsprozess.
  - tournament(k): sample k Individuen zufällig aus mating pool / population (ohne/mit replacement – wähle, was besser zu eurem Code passt; dokumentiere).
  - Wähle das Individuum mit höchster Fitness.

C) „Quick Data“ Logging Option (Gen0 + GenEnd (+ optional))
Ziel: Weniger I/O, trotzdem Start/Ende Vergleich.
- Neue Config:
  - logging_mode: enum {full, quick} (default full / aktuelles Verhalten)
  - quick_keep: u32 (default 2; erlaubt 2 oder 3)
- Umsetzung:
  - FULL: unverändert.
  - QUICK:
    - Speichere vollständige Snapshots für Gen0 und die letzten (quick_keep-1) Generation(en) am Ende (i. d. R. GenEnd, optional GenEnd-1).
    - Während der Simulation: halte einen Ringbuffer für Snapshot-Daten der letzten (quick_keep-1) Generationen oder schreibe temporär und lösche am Schluss; wähle die robusteste Variante für euer Logging.
  - Zusätzlich erlaubt (ohne Verhaltensänderung): pro Generation eine kleine Stats-Datei (CSV/JSON) mit Aggregaten (mean fitness, population size, mean age, mean food, etc.). Diese Stats dürfen immer geschrieben werden, auch im quick mode.

D) Fitness-Mode „Effizienter Sammler“ (Option)
Ziel: Belohnt Kreaturen, die viel Nahrung mit wenig Energieverbrauch sammeln und lange überleben; bestraft Untätigkeit/Zappeln.
- Neue Config:
  - fitness_mode: enum {classic, efficient_collector} (default classic / aktuelles Verhalten)
  - Optional: weights (a,b,c,d,e) als Config-Felder, aber setze sinnvolle Defaults, so dass man nicht zwingend konfigurieren muss.
- Tracking pro Kreatur (falls nicht vorhanden):
  - food_collected (u32)
  - energy_spent (f32) oder sum(abs(delta_energy_neg)) – konsistent mit eurem Energiemodell
  - survival_steps (u32)
  - idle_steps (u32): definiere „idle“ als z. B. Bewegung < epsilon oder action = noop (je nach Sim)
  - jitter_score (u32 oder f32): z. B. viele Richtungswechsel/Actions ohne Wegstrecke
- Fitness-Formel (Startvorschlag; dokumentiere):
  fitness = a*food_collected
          + b*(food_collected / (energy_spent + eps))
          + c*survival_steps
          - d*idle_penalty
          - e*jitter_penalty
  - Idle-Handling: bestrafe erst ab einer Toleranz (z. B. idle_streak > N) ODER nur wenn in der Idle-Phase keine Nahrung aufgenommen wurde.
  - Classic Mode muss exakt der bestehende Codepfad bleiben.

E) Gehirngröße anpassen (mehr Layers) – evolvierbar, opt-in
Ziel: Brain-Architektur soll optional evolvierbar sein.
- Neue Config:
  - brain_mode: enum {fixed, evolvable} (default fixed)
  - max_hidden_layers: u32 (default z. B. 4)
  - layer_min_neurons: u32 (default z. B. 4)
  - layer_max_neurons: u32 (default z. B. 64)
- Umsetzung:
  - Wenn fixed: nutze bestehende Architektur unverändert.
  - Wenn evolvable:
    - DNA/Genome enthält layers: Vec<usize> (z. B. [input, h1, h2, output]).
    - Mutation-Operatoren:
      1) add_hidden_layer (wenn < max_hidden_layers)
      2) remove_hidden_layer (wenn > 1 hidden)
      3) resize_hidden_layer (+/- delta, clamp min/max)
    - Stelle sicher, dass Brain-Konstruktion aus layers robust ist.

F) Crossover/Rekombination bei Fortpflanzung (Option)
Ziel: Nicht nur Klonen+Mutation, sondern echte Rekombination der Brain-Weights.
- Neue Config:
  - crossover_mode: enum {none, layer, blend} (default none)
  - arch_inherit: enum {fitter, random} (default fitter) – nur relevant wenn Eltern unterschiedliche Architektur haben (bei evolvable).
- Umsetzung:
  - Wenn crossover=none: bestehendes Verhalten beibehalten.
  - Wenn crossover aktiv:
    - Architektur des Kindes:
      - Wenn brain_mode=fixed: Architektur identisch.
      - Wenn evolvable:
        - arch_inherit=fitter: Kind übernimmt layers vom fitteren Parent.
        - arch_inherit=random: Kind übernimmt layers von zufällig gewähltem Parent.
    - Gewicht-Crossover:
      - layer: pro Layer/Matrix wähle komplett Parent A oder B.
      - blend: child = alpha*A + (1-alpha)*B, alpha pro Layer zufällig im [0,1].
    - Shape-Mismatch:
      - Nur überlappende Layer kombinieren.
      - Nicht-overlappende Layer: vom Architektur-Parent kopieren oder neu initialisieren (bevorzugt kopieren für Stabilität); dokumentiere.
  - Danach normale Mutation anwenden (wenn eure Pipeline das so macht). Achte auf Reihenfolge und dokumentiere sie.

CLI FLAGS (in bestehende Config einhängen, Namen ggf. eurem Stil anpassen)
- --selection roulette|tournament
- --tournament-k <u32>
- --fitness classic|efficient
- --logging full|quick
- --quick-keep 2|3
- --food-vision-radius <u32>     (0=aus)
- --brain fixed|evolvable
- --max-hidden-layers <u32>
- --layer-min-neurons <u32>
- --layer-max-neurons <u32>
- --crossover none|layer|blend
- --arch-inherit fitter|random
(Seed/Generations etc. bleiben wie bereits vorhanden.)

INTEGRATION / CODE-QUALITÄT
- Halte Änderungen lokal und modular (z. B. selection.rs, fitness.rs, crossover.rs, perception.rs), aber passe dich der existierenden Struktur an.
- Keine Magic Numbers ohne Konstanten/Config.
- Gute Fehlermeldungen bei ungültigen Flag-Kombinationen (z. B. crossover!=none aber brain_mode=fixed ist ok; tournament-k < 2 -> clamp oder error; quick_keep nur 2..3 -> error).
- Dokumentiere neue Optionen in README oder CLI --help Text.

TESTS / CHECKS (minimum)
- Unit-Tests oder kleine deterministische Checks für:
  - Tournament selection: wählt bestes aus sample.
  - Crossover: erzeugt korrekte Shapes, kein Panic bei mismatch.
  - FOV: Food außerhalb Radius wird nicht erkannt; innerhalb wird erkannt.
- Build: cargo fmt, cargo clippy (wenn im Projekt genutzt), cargo test.

DEFINITION OF DONE
- `cargo run -- ...` ohne neue Flags verhält sich wie vorher (keine Regression).
- Alle neuen Flags funktionieren, sind dokumentiert und ändern nur bei Aktivierung das Verhalten.
- Simulation läuft stabil über viele Generationen ohne Shape/Panic/Index-Fehler.
- Quick-Logging schreibt nur Gen0 + GenEnd (und optional GenEnd-1), Stats pro Gen optional immer.

LIEFERUMFANG
- Implementiere Code + aktualisiere Config/CLI + Logging + Dokumentation + Tests.
- Wenn du Stellen findest, wo eine „Default-gute Idee“ extrem sinnvoll ist (rein strukturell/telemetry, ohne Behavior Change), darfst du sie hinzufügen, aber erkläre sie kurz in README/Kommentar.



3. Prompt:

Du arbeitest auf einem **bestehenden Rust-Projekt** namens **EvoBrain** (headless evolutionary grid simulation mit NN-Agenten). WICHTIG: **Baue auf dem bestehenden Code auf und ergänze ihn nur.** Nichts neu “from scratch” schreiben, keine Architektur komplett umwerfen. Nutze die bereits vorhandene Config/Cargo-Parsing-Struktur und das bestehendeTE Logging-Setup (Quick/Full, Snapshots etc.), und erweitere es um **Generations-Metrics-Erfassung mit selektiver Generationsauswahl**.

## Ziel
Implementiere ein System, das pro Generation einen **GenerationReport** (Summary + optional Details) erzeugt und als **JSON** (pro geloggter Generation) sowie als **CSV** (eine Zeile pro geloggter Generation) speichert. Der Nutzer soll per CLI/Config festlegen können, **welche Generationen geloggt werden** (z.B. nur jede 10., nur bestimmte Bereiche, nur einzelne Generationen).

Das System muss:
- pro Generation Metriken berechnen (siehe Liste unten)
- nur dann schreiben, wenn die Generation laut Selector ausgewählt ist
- Counters pro Generation immer korrekt führen und am Ende jeder Generation resetten (auch wenn nicht geloggt wird)
- in “full logging” zusätzlich Per-Creature Daten (Top 10 oder alle; default Top 10) speichern

## Erweiterungen (nicht neu bauen)
1) Neue CLI/Config Optionen (build on existing parsing):
   - `--log-gens <SPEC>`: Auswahl, welche Generationen geloggt werden
   - `--full-log-gens <SPEC>`: optional; Auswahl, in welchen Generationen zusätzlich Full-Details geschrieben werden
   - `--log-mode <quick|full>` existiert ggf. schon — nutze es; sonst ergänzen, aber minimal.
   - `--run-id <STRING>` optional (falls nicht vorhanden); wenn nicht gesetzt: generiere automatisch (z.B. timestamp + seed)
   - `--seed <u64>` existiert ggf. schon; nutze es.

   SPEC Format:
   - `all` => alle Generationen
   - `none`/leer => keine
   - `N` => einzelne Generation (z.B. `25`)
   - `A-B` => Range inkl. (z.B. `10-50`)
   - `A-B/S` => Range inkl. mit Schrittweite (z.B. `0-200/10`)
   - Komma-getrennte Kombinationen: `0-2,0-500/10,250,300-320`
   Parser muss robust sein und sinnvolle Fehlermeldungen liefern.

2) Output Ordnerstruktur (pro Run):
   Lege pro Simulation-Run einen Output-Ordner an (existiert ggf. schon; erweitere konsistent):
   `runs/<run_id>/`
     - `manifest.json` (einmalig pro Run)
     - `generations.csv` (Append, header nur einmal)
     - `gen_0000.json`, `gen_0010.json`, … (nur geloggte Generationen)

3) Reproduzierbarkeit:
   - `run_id`, `seed`, kompletter Config Snapshot ODER `config_hash` (am besten beides: config_hash in Report + manifest.json enthält vollen config snapshot)
   - optional `git_commit` im Report/Manifest (wenn verfügbar). Falls nicht verfügbar: `null`.

## Zu erfassende Daten
### Population / Erfolg
- generation, steps_per_gen, population_size
- fitness_best, fitness_mean, fitness_median, fitness_std
- food_eaten_total, food_eaten_mean
- survival_steps_mean (oder mean age)
- reproductions_total

### Netzwerk-Komplexität
- params_mean, params_median, params_best
- optional: layers_mean, hidden_mean
- zusätzlich für grobe Diversity: params_std

### Diversity grob
- fitness_iqr (Quartilsabstand)

### Evolution-Operatoren
- mutation_rate, mutation_sigma
- crossover_rate
- selection_mode (roulette/tournament) + ggf tournament_k

### Reproduzierbarkeit
- seed, run_id
- config_hash + manifest mit config snapshot
- optional git_commit

### Optional (nur bei Full-Details)
Per-Creature Daten (default Top 10 nach Fitness; optional “all” falls sinnvoll/leicht):
- id, fitness, food_eaten, age (oder survival_steps), params, layers (optional)

## Implementationsanforderungen
A) Datenmodell (serde):
- Erstelle `GenerationReport` + `IndividualSummary` structs (Serialize/Deserialize).
- Felder wie oben; optionale Felder via `Option<>`.
- Für CSV: speichere nur flache Summary-Felder in `generations.csv` (ohne `individuals`). JSON darf `individuals` enthalten.
  => Lösung: entweder separate `GenerationReportCsvRow` oder CSV-Writer, der `individuals` ignoriert.

B) Selector:
- Implementiere `GenSelection` (All/None/Parts) + Parser für SPEC.
- `matches(gen: u32) -> bool`.
- Zusätzlich: `full_matches(gen)` (wenn `--full-log-gens` nicht gesetzt: Full-Details werden abhängig von global mode geschrieben; wenn gesetzt: Full-Details nur in den gematchten Generationen).

C) Collector:
- Implementiere einen `MetricsCollector`, der:
  - pro Step / pro Event Counter sammelt: `food_eaten_total`, `reproductions_total`
  - am Ende der Generation aus einer Liste von finalen Creature-Stats Metriken berechnet
  - danach pro Generation resetten kann

- Stats-Funktionen:
  - mean, median (sortiert), std (Population std reicht), iqr (Q3 - Q1; auf sortierten Werten; simple Quartil-Index-Methode ist okay)
  - robust bei leerer Liste (sollte in der Simulation nicht passieren, aber defensiv)

D) “CreatureFinalStats” Mapping:
- Finde im bestehenden Code, wo am Ende einer Generation die Population bewertet wird.
- Erzeuge pro Creature ein `CreatureFinalStats` (oder verwende bestehende Stats, wenn bereits vorhanden) mit:
  - id (falls vorhanden; sonst index-basierter u64)
  - fitness
  - food_eaten
  - age/survival_steps
  - params (Parameterzahl des NN)
  - layers (optional)
  - hidden (optional)

Parameterzahl:
- Nutze vorhandene NN-Struktur; implementiere eine Methode `fn param_count(&self) -> u32` am Brain/Netz-Objekt.
- Falls Architektur-Evolution existiert, muss `param_count()` die aktuelle Architektur korrekt abbilden.

E) Writer:
- Schreibe:
  - `manifest.json` einmalig pro Run: beinhaltet run_id, seed, config snapshot, config_hash, optional git_commit, timestamp.
  - `gen_{:04}.json` pro geloggter Generation: `GenerationReport` (mit optional `individuals`)
  - `generations.csv` Append: eine Zeile pro geloggter Generation, Header nur beim ersten Mal.
- Nutze `serde_json`, `csv` crate. Für Hash: `blake3` (oder falls bereits vorhanden: vorhandene Hash-Lösung nutzen; aber blake3 ist ok).
- Achte auf stabile floats (normal f32 reicht).

F) Integration in Simulation Loop (wichtig: nur ergänzen!)
- Finde die Stelle “Ende einer Generation”.
- Dort:
  1) Berechne/erzeuge `creatures_final_stats`
  2) `should_log = log_gens.matches(gen)`
  3) `should_full = (log_mode==Full) && (full_log_gens.matches(gen) wenn gesetzt sonst true)`
  4) Wenn `should_log`: baue Report; setze `individuals` je nach `should_full` (sonst None)
  5) Writer schreibt JSON + CSV
  6) Collector resetten (immer)
- Hooke Collector-Counter in bestehende Logik:
  - Wenn irgendwo Food-Essen passiert: `collector.on_food_eaten(1 oder amount)`
  - Wenn Reproduktion passiert: `collector.on_reproduction()`
  - Falls es bereits Event-Logging gibt: integriere minimal dort.

G) Kompatibilität / Minimale Änderungen
- Keine Breaking Changes: Default-Verhalten soll sinnvoll sein:
  - Wenn `--log-gens` nicht gesetzt: default = `all` ODER (besser) `0-999999/1` -> also all
  - Wenn `--full-log-gens` nicht gesetzt: Full-Details werden nach `--log-mode` entschieden
- Bestehende Snapshot-Exports nicht entfernen; nur ergänzen.

H) Tests (minimal, aber wichtig)
- Unit tests für:
  - `parse_gen_selection` für: `all`, `none`, `25`, `10-50`, `0-200/10`, `0-2,0-500/10,250,300-320`
  - `matches()` correctness
- Optional: Test für `config_hash` deterministisch (gleiche config => gleicher hash)

## Deliverables (konkret)
1) Neue/angepasste Module/Files:
   - z.B. `src/metrics/mod.rs`, `src/metrics/selector.rs`, `src/metrics/report.rs`, `src/metrics/writer.rs`
   - oder passend zu bestehender Struktur (halte Stil/Ordner des Projekts ein).
2) Erweiterte Config/CLI parsing: neue Flags, in Config-Struktur aufgenommen.
3) Integration im Simulation-Runner (Ende Generation) inkl. Writer-Call.
4) `manifest.json` + `generations.csv` Output implementiert.

## Coding Style
- Halte dich an vorhandene Code-Styles, Naming, Error-Handling (anyhow/thiserror falls vorhanden).
- Verwende `serde` derive.
- Kein unnötiges Refactoring.

## Akzeptanzkriterien
- Simulation läuft unverändert durch, wenn Logging aktiv ist.
- Bei `--log-gens "0-100/10"` entstehen JSONs nur für gen 0,10,20,…,100 und CSV enthält genau diese Zeilen.
- `--log-mode full` schreibt `individuals` (Top 10) in JSON; `--log-mode quick` nicht.
- `--full-log-gens` begrenzt Full-Details auf die gematchten Generationen.
- `manifest.json` wird pro Run genau einmal geschrieben.
- Tests für GenSelection Parser laufen grün.

Los: Implementiere das nun im bestehenden Projekt. Suche zuerst die bestehenden Stellen für Config parsing, Simulation loop, Reproduktion/Food Events und NN-Struktur für param_count(), und ergänze dort minimal.