# Invasia Simulation Optimization Summary

## Performance Improvements

### Benchmark Results
- **Before Optimization**: 5.96 Hz (2.5% of 240 Hz target)
- **After Optimization**: 96.80 Hz (40.3% of 240 Hz target)
- **Improvement**: **16.2x faster** (1,524% improvement)

### Optimization Techniques Applied

#### 1. Fixed-Capacity Inline Array Grid
**Changed**: Replaced `HashMap<(i32, i32), Vec<usize>>` with fixed-size array-based grid
- Grid size: 128×128 = 16,384 cells (heap-allocated Vec with pre-allocated capacity)
- Max entities per cell: 64 (stored inline within each cell tuple)
- **Impact**: Eliminated repeated heap allocations on every rebuild, improved cache locality through inline storage

#### 2. Pre-allocated Reusable Buffers
**Added**: Persistent buffers in `Simulation` struct:
- `neighbor_buffer`: For spatial queries
- `snapshot_buffer`: For entity snapshots
- `resource_transfers`: For death processing
- `dead_indices`: For tracking dead entities
- **Impact**: Eliminated repeated allocations in hot loop

#### 3. Index-Based Entity Lookups
**Changed**: Use indices instead of IDs for O(1) lookups
- Resource transfers now use `(usize, f32, f32)` instead of `(u32, f32, f32)`
- Dead entity processing uses direct array indexing
- **Impact**: Eliminated O(n) `find()` operations

#### 4. Removed Redundant ID Checks
**Changed**: EntitySnapshot no longer stores `id` field
- Only stores: `position_x`, `position_y`, `state`, `military_strength`
- Neighbor queries use indices directly
- **Impact**: Reduced memory footprint, fewer comparisons

#### 5. Optimized Distance Calculations
**Changed**: Use squared distance to avoid `sqrt()`
- Compare `dist_sq < 100.0` instead of `dist < 10.0`
- **Impact**: Eliminated expensive sqrt operations in inner loops

#### 6. Early Returns for Dead Entities
**Added**: Early exit in `update()` for dead entities
- **Impact**: Skips unnecessary computation for dead entities

#### 7. Deterministic Ordering
**Fixed**: Sort country IDs before processing in DecisionSystem
- Ensures reproducible results with same seed
- **Impact**: All tests pass, determinism verified

#### 8. Compiler Optimizations
**Changed**: Cargo.toml release profile:
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
```
- **Impact**: Better inlining, dead code elimination, and optimization across modules

#### 9. Unsafe Optimizations
**Added**: `get_unchecked` for array access in hot paths
- Applied only where bounds are guaranteed by loop structure
- **Impact**: Eliminated bounds checking overhead

## Architecture Changes

### Before
```
Simulation
  ├─ entities: Vec<AiEntity>
  └─ grid: SpatialGrid
       └─ cells: HashMap<(i32,i32), Vec<usize>>  ← Heap allocations every rebuild!
```

### After
```
Simulation
  ├─ entities: Vec<AiEntity>
  ├─ grid: SpatialGrid  
  │    └─ cells: Vec<([usize; 64], usize)>        ← Pre-allocated with inline storage
  ├─ neighbor_buffer: Vec<usize>                  ← Reusable
  ├─ snapshot_buffer: Vec<EntitySnapshot>         ← Reusable
  ├─ resource_transfers: Vec<(usize, f32, f32)>  ← Reusable
  ├─ dead_indices: Vec<usize>                     ← Reusable
  └─ attacker_search_buffer: Vec<usize>           ← Reusable (separate from neighbor_buffer)
```

## Deterministic AI Decisions

### Implementation
The AI decision scoring system is **fully deterministic** when given a seed:

1. **Pure Mathematical Functions**: All decisions use linear algebra and lookup tables
   - Sigmoid LUT: `σ(x) = 1 / (1 + e^(-x))`
   - Log-ratio LUT: `ln(force_ratio)`
   - Distance kernel: `K(d) = exp(-decay * d)`
   - Discount factors: `d^h`

2. **Argmax Selection**: Best action chosen by `argmax(score)`
   - Score = `Σ (weight_i * component_i)`
   - Weights are adaptive but deterministic based on state
   - No randomness in selection

3. **Deterministic Ordering**: HashMap iterations sorted by ID
   - Ensures consistent processing order
   - Test `test_determinism_multiple_runs` passes

## Remaining Bottlenecks

Based on profiling, the remaining time is spent in:
1. **Snapshot Creation** (~10-15%): Copying entity data into snapshots
2. **Spatial Grid Rebuild** (~15-20%): Rebuilding grid every tick
3. **Entity Updates** (~50-60%): Main computation loop
4. **Death Processing** (~5-10%): Finding nearest attackers

## Further Optimization Opportunities

To reach 240 Hz (2.5x more improvement needed):

1. **SIMD Operations**: Use explicit SIMD for distance calculations
2. **Parallel Processing**: Multi-thread entity updates (requires rayon or similar)
3. **Incremental Spatial Grid**: Don't rebuild entirely, update changed cells only
4. **Entity Pooling**: Reuse dead entity slots instead of marking them dead
5. **Reduced Precision**: Consider f32 → f16 for some calculations
6. **Batched Updates**: Process entities in groups for better cache utilization

## Validation

All tests pass including:
- ✅ Entity creation and update
- ✅ Combat and damage calculation
- ✅ Death and resource transfer
- ✅ Determinism (same seed → same results)
- ✅ Decision system scoring
- ✅ Adaptive weights and marginal values
- ✅ Benchmark (10K entities at 96.8 Hz)

## Conclusion

We achieved a **16.2x performance improvement** through:
- Fixed-capacity inline array grid with provable bounds
- Pre-allocated reusable buffers (including separate buffers for different purposes)
- Index-based lookups
- Elimination of redundant operations
- Compiler optimizations
- Debug assertions for unsafe code
- Cell overflow detection and warnings

The system is now **fully deterministic** and processes 10,000 entities at nearly **100 Hz**, a significant improvement from the initial 6 Hz. Further gains to reach 240 Hz would require more fundamental architectural changes such as SIMD or parallelization.
