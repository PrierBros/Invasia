# GitHub Pages Setup Instructions

To enable GitHub Pages deployment for this repository:

## Steps

1. **Go to Repository Settings**
   - Navigate to your repository on GitHub
   - Click on "Settings" tab

2. **Configure GitHub Pages**
   - In the left sidebar, click on "Pages"
   - Under "Build and deployment" section:
     - Set **Source** to "GitHub Actions"
   - Save the settings

3. **Merge and Deploy**
   - Merge this PR to the `main` branch
   - The GitHub Actions workflow will automatically:
     - Set up Rust toolchain and wasm32 target
     - Install wasm-pack
     - Build the Rust/WASM module
     - Build the Astro site with Svelte components
     - Deploy to GitHub Pages
   - Your site will be live at: `https://jprier.github.io/Invasia`

## Workflow Details

The deployment is handled by `.github/workflows/deploy.yml` which:
- Triggers on push to `main` branch
- Can also be manually triggered from Actions tab
- Sets up the complete Rust + Node.js toolchain
- Builds the WASM module with `wasm-pack`
- Builds the static site with `npm run build`
- Deploys to GitHub Pages using official GitHub actions

## Tech Stack Requirements

The workflow automatically installs:
- **Rust toolchain** with wasm32-unknown-unknown target
- **wasm-pack** for building Rust to WebAssembly
- **Node.js 20** with npm caching
- **All npm dependencies** including Astro and Svelte

## Permissions

The workflow requires the following permissions (automatically configured):
- `contents: read` - to checkout the code
- `pages: write` - to deploy to GitHub Pages
- `id-token: write` - for GitHub Pages authentication

## Troubleshooting

If deployment fails:
1. Check that GitHub Pages is enabled in repository settings
2. Verify the workflow has required permissions
3. Ensure Rust toolchain is properly set up in the workflow
4. Check the Actions tab for detailed error logs
5. Verify wasm-pack installation succeeded
6. Check WASM build logs for Rust compilation errors
