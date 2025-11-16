# Testing Guide for Invasia

This document describes the testing infrastructure for the Invasia project, including integration tests for performance benchmarking and end-to-end tests for UI validation.

## Test Infrastructure

### Integration Tests (Rust/WASM)

The Rust integration tests are located in `wasm/src/lib.rs` and validate the core simulation logic.

**Key Test: 10,000 Element Benchmark**

The `test_benchmark_10000_elements_at_240hz` test benchmarks the simulation with 10,000 entities:

- **Purpose**: Validate that the simulation can update all 10,000 elements
- **Target**: 240 Hz (120 FPS × 2 ticks per frame)
- **Current Performance**: ~5.34 Hz in release mode (2.2% of target)
- **What it validates**:
  - All 10,000 entities are created and updated
  - Entities transition between states (Active, Resting, Moving, Idle, Dead)
  - No entities are lost during updates
  - Performance metrics are captured and reported

**Running Integration Tests:**

```bash
# Run all Rust tests (unit tests only, benchmarks are ignored)
npm run test:wasm

# Run benchmark tests (includes ignored tests)
npm run test:wasm:bench
```

**Expected Output:**

```
=== Benchmark: 10,000 Elements at 240 Hz Target ===
Entity count: 10000
Target tick rate: 240 Hz
Target time per tick: 4.17 ms

Warming up...
Running 100 ticks for benchmark...

--- Results ---
Total time for 100 ticks: 18711.74 ms (18.71 s)
Average time per tick: 187.12 ms
Achieved tick rate: 5.34 Hz
Target tick rate: 240 Hz
Performance ratio: 2.2% of target

--- Entity States ---
Active: 3172, Resting: 0, Moving: 0, Idle: 1826, Dead: 5002

✓ Benchmark COMPLETED:
  - Successfully updated all 10000 entities
  - Achieved 5.34 Hz (2.2% of 240 Hz target)
```

### E2E Tests (Playwright)

The E2E tests are located in `e2e/simulation-performance.spec.ts` and test the application through the UI as a user would experience it.

**Key Tests:**

1. **Basic Page Load Test**
   - Verifies the simulation page loads correctly
   - Checks for essential UI elements (h1, buttons, inputs)

2. **10,000 Entity Performance Test**
   - Sets entity count to 10,000 via the UI
   - Starts the simulation
   - Measures FPS using `requestAnimationFrame`
   - Validates the page remains responsive (30+ FPS)
   - Takes screenshots for visual verification

3. **UI Elements Validation**
   - Verifies entity count input exists
   - Checks simulation controls are present
   - Validates page title and metadata

**Running E2E Tests:**

```bash
# Install browsers (first time only)
npx playwright install chromium

# Build WASM module (required before tests)
npm run build:wasm

# Run E2E tests (starts dev server automatically)
npm run test:e2e

# Run with UI mode for debugging
npm run test:e2e:ui

# Run in headed mode to see the browser
npm run test:e2e:headed
```

## Test Scripts

All test scripts are defined in `package.json`:

| Script | Description |
|--------|-------------|
| `npm run test` | Run all tests (WASM unit + E2E) |
| `npm run test:wasm` | Run Rust unit tests |
| `npm run test:wasm:bench` | Run performance benchmark tests |
| `npm run test:e2e` | Run Playwright E2E tests |
| `npm run test:e2e:ui` | Run E2E tests with interactive UI |
| `npm run test:e2e:headed` | Run E2E tests in headed browser mode |

## Performance Notes

### Current Performance Bottleneck

The simulation currently achieves ~5.34 Hz with 10,000 entities in release mode, which is significantly below the 240 Hz target. This is due to:

1. **Spatial Grid Complexity**: O(n) neighbor queries per entity
2. **Combat Calculations**: Distance calculations for all nearby entities
3. **Resource Transfers**: Finding nearest attacker for dead entities

### Optimization Opportunities

To achieve closer to 240 Hz, consider:

1. **Parallel Processing**: Use rayon for parallel entity updates
2. **Spatial Indexing**: Optimize the spatial grid structure
3. **SIMD Operations**: Use SIMD for distance calculations
4. **Reduce State Transitions**: Minimize entity state changes
5. **WebAssembly Threading**: Enable multi-threading in WASM

## CI/CD Integration

To integrate these tests into GitHub Actions:

```yaml
- name: Test WASM
  run: npm run test:wasm

- name: Benchmark WASM (10k entities)
  run: npm run test:wasm:bench

- name: Install Playwright Browsers
  run: npx playwright install --with-deps chromium

- name: Run E2E Tests
  run: npm run test:e2e
```

## Troubleshooting

### WASM Tests Fail with "cannot find module"
- Solution: Ensure you're in the `wasm` directory or use `npm run test:wasm`

### E2E Tests Timeout
- Solution: Increase timeout in `playwright.config.ts`
- Check that dev server is accessible at `http://localhost:4321/Invasia`

### Performance Tests Are Slow
- Ensure you're running with `--release` flag for Rust tests
- Debug builds are much slower (~30x) than release builds

## Screenshots

E2E tests automatically capture screenshots on failure. Screenshots are saved to:
- `/tmp/simulation-*.png` during test execution
- `test-results/` directory (gitignored)

## Test Coverage

Current test coverage:
- ✅ Entity creation and initialization
- ✅ Entity state transitions
- ✅ Combat mechanics
- ✅ Resource transfers
- ✅ Death handling
- ✅ Performance benchmarking (10k entities)
- ✅ UI rendering and responsiveness
- ✅ Page load and navigation
- ✅ Simulation controls

## Future Enhancements

- [ ] Add visual regression tests for UI
- [ ] Test with different entity counts (100, 1k, 5k, 10k, 50k)
- [ ] Measure memory usage during simulation
- [ ] Test on different browsers (Firefox, Safari)
- [ ] Add performance profiling
- [ ] Test WebGL rendering if implemented
