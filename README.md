# Invasia

A modern website built with [Astro](https://astro.build), featuring **Svelte with TypeScript** for DOM manipulation and **Rust compiled to WebAssembly** for high-performance logic and memory handling.

## 🚀 Live Demo

The site is automatically deployed to GitHub Pages at: https://jprier.github.io/Invasia

## ✨ Features

- **Hello World Landing Page** - Beautiful, responsive design with gradient background
- **Rust + WebAssembly Backend** - All counter logic and memory management handled by Rust compiled to WASM
- **AI Decision Scoring System** - Advanced strategic AI for country-based simulation games
- **Svelte + TypeScript Frontend** - Reactive UI components with type safety for DOM manipulation
- **Interactive Counter** - Demonstrates seamless integration between Svelte UI and Rust/WASM logic
- **Static Site Generation** - Fast loading times with pre-rendered HTML
- **Automated Deployment** - GitHub Actions workflow for continuous deployment to GitHub Pages

## 🛠️ Tech Stack

### Frontend
- **[Astro](https://astro.build)** - Static site framework with islands architecture
- **[Svelte](https://svelte.dev)** - Reactive UI components
- **[TypeScript](https://www.typescriptlang.org/)** - Type-safe DOM manipulation

### Backend/Logic
- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[WebAssembly](https://webassembly.org/)** - Binary instruction format for web
- **[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)** - Rust/WASM ↔ JavaScript interop

## 📋 Prerequisites

- Node.js 20.x or higher
- npm 10.x or higher
- Rust 1.70 or higher
- wasm-pack

## 🔧 Development

### Installation

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install Node.js dependencies
npm install
```

### Local Development

```bash
# Build the WASM module
npm run build:wasm

# Start the development server
npm run dev

# Or run both steps together
npm run build:wasm && npm run dev
```

The dev server will be available at `http://localhost:4321/Invasia`

### Building for Production

```bash
# Build WASM module
npm run build:wasm

# Build the static site
npm run build

# Preview the production build
npm run preview
```

### Testing

```bash
# Run Rust tests
npm run test:wasm

# Or directly with cargo
cd wasm-counter && cargo test
```

## 📦 Project Structure

```
/
├── public/
│   └── favicon.svg          # Site favicon
├── src/
│   ├── components/
│   │   └── Counter.svelte   # Svelte counter component (TypeScript)
│   ├── pages/
│   │   └── index.astro      # Main page
│   └── wasm/                # Generated WASM files (excluded from git)
│       ├── wasm_counter.js
│       ├── wasm_counter_bg.wasm
│       └── ...
├── wasm-counter/            # Rust/WASM source
│   ├── src/
│   │   ├── lib.rs           # Counter logic in Rust
│   │   └── decision_scoring/ # AI Decision Scoring System
│   │       ├── mod.rs       # Module definitions
│   │       ├── luts.rs      # Lookup tables (sigmoid, log-ratio, etc.)
│   │       ├── country.rs   # Country state and adaptive weights
│   │       ├── actions.rs   # Action types and candidate pruning
│   │       ├── scoring.rs   # Six-channel decision scoring
│   │       ├── world.rs     # World state and tick execution
│   │       └── README.md    # AI system documentation
│   └── Cargo.toml           # Rust project configuration
├── .github/
│   └── workflows/
│       ├── deploy.yml       # GitHub Pages deployment
│       └── copilot-setup-steps.yml  # Copilot environment setup
├── astro.config.mjs         # Astro configuration
├── package.json             # Node.js dependencies and scripts
└── tsconfig.json            # TypeScript configuration
```

## 🏗️ Architecture

### Data Flow

1. **Rust/WASM Module** (`wasm-counter/src/lib.rs`)
   - Implements `Counter` struct with increment/decrement/reset logic
   - Compiled to WebAssembly for near-native performance
   - Manages memory safely using Rust's ownership system

2. **Svelte Component** (`src/components/Counter.svelte`)
   - Loads WASM module asynchronously on mount
   - Manages UI state reactively with TypeScript
   - Calls Rust functions through WASM bindings
   - Updates DOM efficiently with Svelte's reactivity

3. **Astro Page** (`src/pages/index.astro`)
   - Server-renders the page structure
   - Hydrates Svelte component on the client
   - Provides optimal loading performance

### Why This Stack?

- **Rust + WASM**: Memory safety, zero-cost abstractions, and near-native performance for complex logic
- **Svelte + TypeScript**: Minimal runtime overhead, reactive DOM updates, and type safety
- **Astro**: Optimal static site generation with islands architecture for selective hydration

## 🌐 Deployment

The site is automatically deployed to GitHub Pages when changes are pushed to the `main` branch. The deployment workflow:

1. Sets up Rust toolchain and wasm32 target
2. Installs wasm-pack
3. Builds the WASM module
4. Installs Node.js dependencies
5. Builds the static site
6. Deploys to GitHub Pages

To enable GitHub Pages for your fork:
1. Go to repository Settings → Pages
2. Set Source to "GitHub Actions"
3. Push to main branch to trigger deployment

## 📝 License

See [LICENSE](LICENSE) file for details.