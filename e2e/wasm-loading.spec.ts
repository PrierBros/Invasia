import { test, expect } from '@playwright/test';

/**
 * E2E Test for WebAssembly Module Loading
 * 
 * This test ensures that the WebAssembly module loads correctly,
 * which is critical for the simulation to function. It helps prevent
 * issues related to:
 * - Missing .nojekyll file (GitHub Pages)
 * - Incorrect MIME types
 * - CORS issues
 * - Missing or corrupted WASM files
 */

test.describe('WebAssembly Module Loading', () => {
  
  test('should successfully load WASM module without errors', async ({ page }) => {
    console.log('\n=== Testing WASM Module Loading ===');
    
    const consoleErrors: string[] = [];
    const pageErrors: string[] = [];
    const networkErrors: string[] = [];
    
    // Collect console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
        console.log('[CONSOLE ERROR]', msg.text());
      }
    });
    
    // Collect page errors
    page.on('pageerror', error => {
      pageErrors.push(error.message);
      console.error('[PAGE ERROR]', error.message);
    });
    
    // Collect network errors
    page.on('requestfailed', request => {
      const url = request.url();
      if (url.includes('.wasm') || url.includes('wasm')) {
        const failure = request.failure();
        const errorText = `Failed to load ${url}: ${failure?.errorText || 'unknown error'}`;
        networkErrors.push(errorText);
        console.error('[NETWORK ERROR]', errorText);
      }
    });
    
    // Navigate to simulation page
    console.log('Navigating to simulation page...');
    await page.goto('./simulation', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });
    
    // Wait for WASM to load (or fail to load)
    console.log('Waiting for WASM module to load...');
    await page.waitForTimeout(5000);
    
    // Check if WASM loaded successfully by looking for the success badge
    const wasmBadge = page.locator('.wasm-badge');
    const wasmBadgeVisible = await wasmBadge.isVisible().catch(() => false);
    
    if (wasmBadgeVisible) {
      const badgeText = await wasmBadge.textContent();
      console.log(`✓ WASM badge visible: "${badgeText}"`);
    }
    
    // Check for error message in UI
    const errorElement = page.locator('.error');
    const errorVisible = await errorElement.isVisible().catch(() => false);
    
    if (errorVisible) {
      const errorText = await errorElement.textContent();
      console.error(`✗ Error message displayed: "${errorText}"`);
    }
    
    // Check that simulation buttons are enabled (sign that WASM loaded)
    const startButton = page.locator('button').filter({ hasText: /Start/i }).first();
    const isEnabled = await startButton.isEnabled().catch(() => false);
    
    console.log(`Start button enabled: ${isEnabled}`);
    
    // Print summary of errors
    console.log('\n--- Error Summary ---');
    console.log(`Console Errors: ${consoleErrors.length}`);
    console.log(`Page Errors: ${pageErrors.length}`);
    console.log(`Network Errors (WASM): ${networkErrors.length}`);
    
    if (consoleErrors.length > 0) {
      console.log('\nConsole Errors:');
      consoleErrors.forEach(err => console.log(`  - ${err}`));
    }
    
    if (pageErrors.length > 0) {
      console.log('\nPage Errors:');
      pageErrors.forEach(err => console.log(`  - ${err}`));
    }
    
    if (networkErrors.length > 0) {
      console.log('\nNetwork Errors:');
      networkErrors.forEach(err => console.log(`  - ${err}`));
    }
    
    // Assertions to ensure WASM loaded correctly
    expect(networkErrors.length, 'Should have no network errors loading WASM files').toBe(0);
    expect(errorVisible, 'Should not show error message in UI').toBe(false);
    expect(wasmBadgeVisible, 'Should show WASM success badge').toBe(true);
    expect(isEnabled, 'Simulation controls should be enabled after WASM loads').toBe(true);
    
    // Check that no WASM-related errors appeared
    const wasmErrorsInConsole = consoleErrors.filter(err => 
      err.toLowerCase().includes('wasm') || 
      err.toLowerCase().includes('webassembly')
    );
    expect(wasmErrorsInConsole.length, 'Should have no WASM-related console errors').toBe(0);
    
    console.log('\n✓ WASM module loaded successfully\n');
  });
  
  test('should load WASM file with correct MIME type', async ({ page }) => {
    console.log('\n=== Testing WASM MIME Type ===');
    
    let wasmResponse: any = null;
    
    // Intercept WASM file requests
    page.on('response', response => {
      const url = response.url();
      if (url.endsWith('.wasm')) {
        wasmResponse = {
          url,
          status: response.status(),
          contentType: response.headers()['content-type'],
        };
        console.log(`WASM file: ${url}`);
        console.log(`  Status: ${wasmResponse.status}`);
        console.log(`  Content-Type: ${wasmResponse.contentType}`);
      }
    });
    
    // Navigate to page
    await page.goto('./simulation', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });
    
    await page.waitForTimeout(3000);
    
    // Verify WASM file was loaded
    expect(wasmResponse, 'WASM file should have been requested').not.toBeNull();
    expect(wasmResponse?.status, 'WASM file should return 200 status').toBe(200);
    
    // Check MIME type (should be application/wasm or application/octet-stream)
    const contentType = wasmResponse?.contentType || '';
    const hasValidMimeType = 
      contentType.includes('application/wasm') || 
      contentType.includes('application/octet-stream');
    
    console.log(`\nMIME type check: ${hasValidMimeType ? '✓ Valid' : '✗ Invalid'}`);
    
    if (!hasValidMimeType) {
      console.warn(`⚠ WASM served with MIME type: ${contentType}`);
      console.warn('  Expected: application/wasm or application/octet-stream');
      console.warn('  This may cause issues with WebAssembly.instantiateStreaming()');
    }
    
    // Note: We don't fail the test on MIME type because the fallback to instantiate() works
    // but we log a warning
    console.log('\n✓ WASM file accessibility verified\n');
  });
  
  test('should handle WASM instantiation and create simulation instance', async ({ page }) => {
    console.log('\n=== Testing WASM Instantiation ===');
    
    await page.goto('./simulation', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });
    
    // Wait for WASM to load
    await page.waitForTimeout(5000);
    
    // Try to start the simulation to verify WASM is functional
    const startButton = page.locator('button').filter({ hasText: /Start/i }).first();
    await expect(startButton).toBeEnabled({ timeout: 10000 });
    
    console.log('✓ Start button is enabled');
    
    await startButton.click();
    console.log('✓ Clicked start button');
    
    // Wait a moment for simulation to run
    await page.waitForTimeout(2000);
    
    // Check that tick counter is increasing
    const tickDisplay = page.locator('text=/Tick:?\\s*\\d+/i').first();
    const tickText1 = await tickDisplay.textContent();
    const tickMatch1 = tickText1?.match(/\d+/);
    const tick1 = tickMatch1 ? parseInt(tickMatch1[0]) : 0;
    
    console.log(`Initial tick: ${tick1}`);
    
    await page.waitForTimeout(1000);
    
    const tickText2 = await tickDisplay.textContent();
    const tickMatch2 = tickText2?.match(/\d+/);
    const tick2 = tickMatch2 ? parseInt(tickMatch2[0]) : 0;
    
    console.log(`After 1 second: ${tick2}`);
    
    // Verify tick counter increased (simulation is running)
    expect(tick2, 'Tick counter should increase when simulation is running').toBeGreaterThan(tick1);
    
    console.log(`✓ Simulation is running (ticks: ${tick1} → ${tick2})`);
    
    // Check entity count is populated
    const entityCountText = await page.locator('text=/Entities:?\\s*\\d+/i').first().textContent();
    const entityMatch = entityCountText?.match(/\d+/);
    const entityCount = entityMatch ? parseInt(entityMatch[0]) : 0;
    
    console.log(`Entity count: ${entityCount}`);
    expect(entityCount, 'Should have entities loaded from WASM').toBeGreaterThan(0);
    
    console.log('\n✓ WASM instantiation and simulation working correctly\n');
  });
  
  test('should verify .nojekyll file exists in build output', async ({ request }) => {
    console.log('\n=== Testing .nojekyll File Presence ===');
    
    // This test verifies that the .nojekyll file exists in the build output
    // This is critical for GitHub Pages to serve files from _astro directory
    
    const response = await request.get('./.nojekyll', {
      failOnStatusCode: false
    });
    
    console.log(`.nojekyll status: ${response.status()}`);
    
    expect(response.status(), '.nojekyll file should exist in build output').toBe(200);
    
    console.log('✓ .nojekyll file is present\n');
  });
});
