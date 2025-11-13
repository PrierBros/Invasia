# Invasia Architecture

This document describes the technical architecture of Invasia, a modern web application built with Astro, Svelte + TypeScript for the frontend, and Rust + WebAssembly for backend logic.

## Overview

Invasia demonstrates a cutting-edge web architecture that combines:
- **Astro's Islands Architecture** for optimal static site generation
- **Svelte with TypeScript** for reactive, type-safe DOM manipulation
- **Rust compiled to WebAssembly** for high-performance logic and memory management

## Architecture Layers

### 1. Static Site Generation (Astro)

**Purpose**: Server-side rendering and optimal build output

**Technology**: Astro 4.x

**Responsibilities**:
- Build-time page rendering
- Asset optimization and bundling
- Component composition
- Islands architecture implementation
- Static file generation for deployment

**Key Files**:
- `astro.config.mjs` - Astro configuration with Svelte integration
- `src/pages/index.astro` - Main page layout

### 2. UI Layer (Svelte + TypeScript)

**Purpose**: Client-side interactivity and DOM manipulation

**Technology**: Svelte 4.x + TypeScript 5.x

**Responsibilities**:
- Reactive state management
- User interaction handling
- WASM module loading and initialization
- Type-safe component logic
- Efficient DOM updates

**Key Files**:
- `src/components/SimulationTable.svelte` - AI simulation visualization component
- Components use `<script lang="ts">` for TypeScript

**Data Flow**:
1. Component mounts (`onMount`)
2. Dynamically imports WASM module
3. Initializes Rust instances (Simulation or DecisionSystem)
4. Manages reactive state
5. Calls WASM functions on user interaction or at regular intervals
6. Updates UI with Svelte's reactivity

### 3. Logic Layer (Rust + WebAssembly)

**Purpose**: High-performance logic and memory management

**Technology**: Rust 1.70+ with wasm-bindgen

**Responsibilities**:
- AI simulation state management
- Business logic implementation
- Memory-safe operations
- Near-native performance
- Type-safe API exposed to JavaScript

**Key Files**:
- `wasm/src/lib.rs` - Rust implementation with Simulation and DecisionSystem
- `wasm/src/decision_scoring/` - AI decision scoring system modules
- `wasm/Cargo.toml` - Rust project configuration

**API Surface** (exposed via wasm-bindgen):
```rust
// Simulation API
pub struct Simulation {
    entities: Vec<AiEntity>,
    tick: u64,
    running: bool,
}

impl Simulation {
    pub fn new(entity_count: usize) -> Self
    pub fn init(entity_count: usize, tick_rate: u32) -> Self
    pub fn start(&mut self)
    pub fn pause(&mut self)
    pub fn resume(&mut self)
    pub fn reset(&mut self)
    pub fn step(&mut self)
    pub fn update(&mut self)
    pub fn get_snapshot(&self) -> JsValue
    // ... and more
}

// DecisionSystem API (AI Decision Scoring)
pub struct DecisionSystem { /* ... */ }

impl DecisionSystem {
    pub fn new() -> Self
    pub fn init(seed: u64) -> Self
    pub fn add_country(&mut self, id: u32)
    pub fn add_edge(&mut self, from_id: u32, to_id: u32, distance: usize, hostility: f32)
    pub fn tick(&mut self)
    pub fn get_logs(&self) -> JsValue
    pub fn get_world_snapshot(&self) -> JsValue
    // ... and more
}
```

## Build Pipeline

### Development Build

```
1. npm run build:wasm
   └─> wasm-pack build --target web
       ├─> cargo build --target wasm32-unknown-unknown --release
       ├─> wasm-bindgen generates JS bindings
       └─> Outputs to src/wasm/

2. npm run dev
   └─> astro dev
       ├─> Compiles Astro pages
       ├─> Processes Svelte components
       ├─> Bundles with Vite
       └─> Starts dev server
```

### Production Build

```
1. npm run build:wasm
   └─> wasm-pack build --target web
       └─> Optimized WASM binary

2. npm run build
   └─> astro build
       ├─> SSR page generation
       ├─> Svelte component compilation
       ├─> Asset optimization
       ├─> WASM bundling
       └─> Static dist/ output
```

## Runtime Execution Flow

### Initial Page Load

```
1. Browser requests /Invasia
2. Astro serves pre-rendered HTML
3. Static HTML displays immediately
4. JavaScript bundles load
5. Svelte hydrates Counter component
```

### Component Hydration

```
1. SimulationTable.svelte onMount hook fires
2. Dynamic import('../wasm/wasm.js')
3. WASM module initialization
4. Simulation instance created in WASM memory
5. Component state synchronized
6. UI ready for interaction
```

### User Interaction (Simulation)

```
1. User clicks "Start" button
2. Svelte event handler fires
3. Calls simulation.start() (WASM)
4. JavaScript interval starts calling simulation.update()
5. Rust executes entity updates each tick
6. JavaScript periodically fetches snapshot via get_snapshot()
7. Svelte updates reactive state with new entity data
8. DOM updates efficiently with real-time visualization
```

## Performance Characteristics

### Advantages

1. **Fast Initial Load**
   - Pre-rendered HTML
   - No client-side rendering required
   - Minimal JavaScript for hydration

2. **Efficient Updates**
   - Svelte's compiled reactivity (no virtual DOM)
   - Direct DOM manipulation
   - Minimal runtime overhead

3. **High-Performance Logic**
   - WASM executes at near-native speed
   - Rust's zero-cost abstractions
   - Memory safety without garbage collection

4. **Small Bundle Size**
   - WASM binary is compact (~161KB uncompressed for full system)
   - Svelte components compile to minimal JS
   - Tree-shaking removes unused code

### Trade-offs

1. **Build Complexity**
   - Requires Rust toolchain
   - WASM build step adds time
   - Multi-language debugging

2. **Initial WASM Load**
   - WASM module must be downloaded
   - Async initialization required
   - Fallback logic recommended

## Security Considerations

1. **Memory Safety**
   - Rust prevents common vulnerabilities
   - No buffer overflows or use-after-free
   - Type safety across language boundary

2. **WebAssembly Sandboxing**
   - WASM runs in browser sandbox
   - No direct DOM access from Rust
   - Controlled API surface via wasm-bindgen

3. **Type Safety**
   - TypeScript for compile-time checks
   - Rust's strong type system
   - wasm-bindgen validates types at boundary

## Deployment Architecture

### CI/CD Pipeline (GitHub Actions)

```
Trigger: Push to main
  ↓
Setup Rust Toolchain
  ├─> Install rustc
  ├─> Add wasm32 target
  └─> Install wasm-pack
  ↓
Setup Node.js
  └─> Install npm dependencies
  ↓
Build WASM
  └─> wasm-pack build
  ↓
Build Site
  └─> astro build
  ↓
Deploy
  └─> GitHub Pages
```

### Static Hosting

- **Platform**: GitHub Pages
- **Output**: Pure static files
- **CDN**: GitHub's edge network
- **HTTPS**: Automatic via GitHub

## Future Enhancements

Potential improvements to the architecture:

1. **WASM Optimization**
   - Enable wasm-opt for smaller binaries
   - Implement streaming compilation
   - Add WASM caching strategies

2. **Component Library**
   - Extract reusable Svelte components
   - Build component storybook
   - Create design system

3. **Advanced WASM Features**
   - Shared memory for threading
   - SIMD optimizations
   - More complex Rust logic

4. **Performance Monitoring**
   - Add Web Vitals tracking
   - WASM load time metrics
   - Bundle size monitoring

## Development Guidelines

### Adding New WASM Functions

1. Add function to `wasm/src/lib.rs`
2. Mark with `#[wasm_bindgen]` attribute
3. Rebuild: `npm run build:wasm`
4. Use in Svelte component

### Modifying Svelte Components

1. Edit `.svelte` file
2. Maintain TypeScript typing
3. Handle WASM async initialization
4. Ensure proper cleanup in onDestroy for long-running processes
5. Test dev and prod builds

### Testing Strategy

- **Rust**: Unit tests with `cargo test`
- **Svelte**: Component testing (to be added)
- **Integration**: Manual browser testing
- **E2E**: Playwright (to be added)

## Conclusion

This architecture demonstrates modern web development practices:
- Performance through WASM
- Developer experience through TypeScript
- Safety through Rust
- Simplicity through Astro's islands
- Reactivity through Svelte

The result is a fast, safe, and maintainable web application.
