# Invasia

A modern website built with [Astro](https://astro.build) featuring JavaScript Islands for interactive components.

## 🚀 Live Demo

The site is automatically deployed to GitHub Pages at: https://jprier.github.io/Invasia

## ✨ Features

- **Hello World Landing Page** - Beautiful, responsive design with gradient background
- **Interactive Counter (JS Island)** - Demonstrates client-side interactivity with increment/decrement buttons
- **Static Site Generation** - Fast loading times with pre-rendered HTML
- **Automated Deployment** - GitHub Actions workflow for continuous deployment to GitHub Pages

## 🛠️ Development

### Prerequisites

- Node.js 20.x or higher
- npm 10.x or higher

### Local Development

```bash
# Install dependencies
npm install

# Start the development server
npm run dev

# Build for production
npm run build

# Preview the production build
npm run preview
```

The dev server will be available at `http://localhost:4321/Invasia`

## 📦 Project Structure

```
/
├── public/
│   └── favicon.svg          # Site favicon
├── src/
│   └── pages/
│       └── index.astro      # Main page with JS island
├── .github/
│   └── workflows/
│       └── deploy.yml       # GitHub Pages deployment workflow
├── astro.config.mjs         # Astro configuration
├── package.json             # Dependencies and scripts
└── tsconfig.json            # TypeScript configuration
```

## 🌐 Deployment

The site is automatically deployed to GitHub Pages when changes are pushed to the `main` branch. The deployment workflow:

1. Installs dependencies
2. Builds the static site
3. Deploys to GitHub Pages

To enable GitHub Pages for your fork:
1. Go to repository Settings → Pages
2. Set Source to "GitHub Actions"
3. Push to main branch to trigger deployment

## 📝 License

See [LICENSE](LICENSE) file for details.