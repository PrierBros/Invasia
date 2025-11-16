# AI Decision Scoring System

This module implements a comprehensive AI decision-making system for country-based strategy games, following the specification in AI Decision Scoring Spec (v1).

## Overview

The system enables AI countries to make strategic decisions by:
1. Computing a scalar **Decision Score** for each candidate action
2. Choosing the action with the highest score each tick
3. Using adaptive weights that respond to country needs
4. Maintaining deterministic behavior with bounded computational cost

## Core Components

### 1. Lookup Tables (`luts.rs`)

Pre-computed tables for efficient calculation:
- **SigmoidLUT**: Logistic function for probabilities
- **LogRatioLUT**: Force ratio calculations
- **DiscountLUT**: Future value discounting
- **DistanceKernelLUT**: Distance-based threat weighting

### 2. Country State (`country.rs`)

Represents a country with:
- Core stats: military strength, GDP, growth, prestige, morale
- **Adaptive weights** (α, β, γ, δ, κ, ρ) that adjust based on needs
- **Marginal values** for research prioritization
- Edges to neighbors with terrain, distance, hostility
- Border tiles for defensive actions

### 3. Actions (`actions.rs`)

Available action types:
- **Attack**: Military action against a neighbor
- **Invest**: Economic/military development
- **Research**: Technology advancement
- **Diplomacy**: Alliances, pacts, trade agreements
- **Fortify/Move**: Border defense
- **Pass**: Do nothing (baseline)

### 4. Scoring (`scoring.rs`)

Six-channel decision formula:

```rust
Score = α·ΔRes + β·ΔSec + γ·ΔGrowth + δ·ΔPos - κ·Cost - ρ·Risk
```

Where:
- **ΔRes**: Expected resource gain
- **ΔSec**: Threat reduction (via Threat Index)
- **ΔGrowth**: Economic/tech trajectory improvement
- **ΔPos**: Diplomatic/positional advantage
- **Cost**: Immediate costs (casualties, resources, diplomatic)
- **Risk**: Outcome uncertainty penalty

### 5. World State (`world.rs`)

The main coordinator:
- **WorldState**: Manages all countries and relationships
- **DecisionSystem**: Executes the tick contract
- **DecisionLog**: Telemetry for debugging and analysis

## Tick Contract

Each simulation tick follows six steps:

1. **Update weights**: Adjust α-ρ based on resource levels, threats, etc.
2. **Update fields**: Recompute threat indices incrementally
3. **Build shortlist**: Generate top-K candidates per action type
4. **Score actions**: Compute six-channel scores for each candidate
5. **Choose**: Select argmax score (deterministic)
6. **Apply**: Execute chosen actions and update world state

## Usage Example

```rust
use wasm::decision_scoring::*;

// Create system with deterministic seed
let mut system = DecisionSystem::init(12345);

// Add countries
system.add_country(1);
system.add_country(2);

// Add relationships (from, to, distance, hostility)
system.add_edge(1, 2, 1, 0.7);
system.add_edge(2, 1, 1, 0.5);

// Run simulation
for _ in 0..10 {
    system.tick();
}

// Get decision logs
let logs = system.get_logs();
```

## Determinism Guarantee

Given the same:
- Initial world state
- RNG seed
- Sequence of tick calls

The system produces **bit-identical** results across runs, satisfying the spec's determinism requirement.

## Performance Characteristics

- **Per-tick cost**: O(countries × avg_degree × shortlist_size)
- **No global scans**: All computations use local/cached data
- **Bounded candidate lists**: Top-K pruning prevents combinatorial explosion
- **SIMD batch scoring**: `score_actions_batch` fuses dot products using wasm `simd128` when available, with scalar fallback elsewhere (set `RUSTFLAGS="-C target-feature=+simd128"` or an equivalent toolchain flag during wasm builds to activate it)
- **Fixed-point arithmetic**: All LUTs use precomputed tables

## Normalization

All score components are normalized to standard ranges:
- ΔRes, ΔSec, ΔGrowth, ΔPos: [-32, +32]
- Cost, Risk: [0, 16]
- Weights (α-ρ): [2, 16] (integers)

This ensures components have comparable magnitudes for stable scoring.

## Testing

The module includes comprehensive tests:
- **Unit tests**: Each component (LUTs, scoring, actions)
- **Integration tests**: Full tick execution
- **Determinism tests**: Verify identical results across runs
- **Normalization tests**: Ensure bounded ranges
- **Adaptive weights tests**: Verify proper clamping

Run tests with:
```bash
cargo test decision_scoring
```

## WebAssembly Interface

The `DecisionSystem` is exposed to JavaScript via `wasm-bindgen`:

```javascript
import init, { DecisionSystem } from './wasm/wasm.js';

await init();

const system = DecisionSystem.init(42);
system.add_country(1);
system.add_country(2);
system.add_edge(1, 2, 1, 0.5);

system.tick();

const logs = system.get_logs();
const world = system.get_world_snapshot();
```

## Future Enhancements

Potential improvements:
- [x] SIMD optimizations for batch scoring
- [ ] Incremental threat index updates (currently recomputed each tick)
- [ ] Machine learning for weight tuning
- [ ] Advanced diplomacy modeling
- [ ] Terrain and supply line simulation
- [ ] Multi-threaded candidate evaluation
