# 🎯 Roadmap

The project follows semantic versioning and an incremental development process.
Each milestone has a clear objective, acceptance criteria, and engineering requirements.

---

## 🚀 v0.0.1 — Bases

**Focus:** Establish a clean architecture and deterministic procedural generation.

### Goals

- [X] Initialize the Bevy application.
- [X] Design the ECS architecture.
- [X] Implement deterministic seed hashing.
- [X] Generate the first star from a seed.

### Engineering

- [ ] Configure GitHub Actions.
- [ ] Enable `cargo fmt`.
- [ ] Enable `cargo clippy`.
- [ ] Add unit tests for hashing.
- [ ] Configure project documentation.

### Definition of Done

- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] `cargo fmt --check` passes.
- [X] Same seed always produces identical output.
- [ ] README updated.

---

## 🌌 v0.0.2 — Interface

**Focus:** Render generated systems and provide basic navigation.

### Goals

- [X] Render generated stars.
- [X] Render planets.
- [X] Camera controls.
- [ ] Display current seed.
- [ ] Display generation statistics.

### Engineering

- [ ] Integration tests.
- [X] Rendering module isolated from generation logic.
- [ ] Integrate logging.

### Definition of Done

- [ ] Any valid seed is renderable.
- [ ] Camera remains stable at all zoom levels.
- [ ] Documentation updated.

---

## 🪐 v0.0.3 — Planet Generation

**Focus:** Generate richer planetary systems.

### Goals

- [ ] Planet classification.
- [ ] Orbital parameters.
- [ ] Atmospheric generation.
- [ ] Planetary biomes.
- [ ] Moons.

### Engineering

- [ ] Benchmark generation performance.
- [ ] Add property tests.
- [ ] Improve serialization.

### Definition of Done

- [ ] Planet generation remains deterministic.
- [ ] Generation time < 100 ms for a standard system.
- [ ] Benchmarks committed.

---

## 🌍 v0.0.4 — Infinite Universe

**Focus:** Scale procedural generation.

### Goals

- [X] Infinite sector generation.
- [X] Dynamic loading.
- [X] Dynamic unloading.
- [ ] World streaming.

### Engineering

- [ ] Memory profiling.
- [ ] Cache optimization.
- [ ] Parallel generation.

### Definition of Done

- [ ] Stable memory usage.
- [ ] Profiling report available.

---

## 🚀 v0.1.0 — Public Alpha

### Features

- [ ] Save generated systems.
- [ ] Load systems.
- [ ] Export JSON.
- [ ] Export screenshots.

### Engineering

- [ ] API documentation.
- [ ] Release binaries.
- [ ] Changelog.

### Definition of Done

- [ ] Linux builds.
- [ ] Windows builds.
- [ ] Public release.

---

# 🧪 Quality Standards

Every milestone must satisfy the following requirements:

- All tests pass.
- CI is green.
- Documentation is updated.
- No Clippy warnings.
- Code formatted with rustfmt.
- Public APIs documented.

---

# 🛠 Technical Debt

Known improvements that are intentionally postponed.

- Better ECS scheduling.
- GPU instancing.
- Parallel chunk generation.
- Configurable rendering pipeline.
