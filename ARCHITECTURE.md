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
- `src/components/Counter.svelte` - Interactive counter component
- Component uses `<script lang="ts">` for TypeScript

**Data Flow**:
1. Component mounts (`onMount`)
2. Dynamically imports WASM module
3. Initializes Rust Counter instance
4. Manages reactive state
5. Calls WASM functions on user interaction
6. Updates UI with Svelte's reactivity

### 3. Logic Layer (Rust + WebAssembly)

**Purpose**: High-performance logic and memory management

**Technology**: Rust 1.70+ with wasm-bindgen

**Responsibilities**:
- Counter state management
- Business logic implementation
- Memory-safe operations
- Near-native performance
- Type-safe API exposed to JavaScript

**Key Files**:
- `wasm-counter/src/lib.rs` - Rust implementation
- `wasm-counter/Cargo.toml` - Rust project configuration

**API Surface** (exposed via wasm-bindgen):
```rust
pub struct Counter {
    value: i32,
}

impl Counter {
    pub fn new() -> Counter
    pub fn with_value(initial: i32) -> Counter
    pub fn increment(&mut self) -> i32
    pub fn decrement(&mut self) -> i32
    pub fn get_value(&self) -> i32
    pub fn reset(&mut self)
    pub fn set_value(&mut self, value: i32)
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
1. Counter.svelte onMount hook fires
2. Dynamic import('../wasm/wasm_counter.js')
3. WASM module initialization
4. Counter instance created in WASM memory
5. Component state synchronized
6. UI ready for interaction
```

### User Interaction

```
1. User clicks "Increment" button
2. Svelte event handler fires
3. Calls counter.increment() (WASM)
4. Rust executes: self.value.saturating_add(1)
5. Returns new value to JavaScript
6. Svelte updates reactive state
7. DOM updates efficiently
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
   - WASM binary is compact (~22KB uncompressed)
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

1. Add function to `wasm-counter/src/lib.rs`
2. Mark with `#[wasm_bindgen]` attribute
3. Rebuild: `npm run build:wasm`
4. Use in Svelte component

### Modifying Svelte Components

1. Edit `.svelte` file
2. Maintain TypeScript typing
3. Handle WASM async initialization
4. Test dev and prod builds

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
