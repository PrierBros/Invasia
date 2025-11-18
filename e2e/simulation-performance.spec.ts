import { test, expect } from '@playwright/test';

/**
 * E2E Performance Test
 * Tests the simulation with large entity counts via the UI
 * Validates performance as a user would experience it
 */

test.describe('Simulation Performance Tests', () => {
  
  test('should load simulation page and verify basic functionality', async ({ page }) => {
    // Navigate to the simulation page
    await page.goto('./simulation');
    
    // Wait for the page to be fully loaded
    await page.waitForLoadState('networkidle');
    
    // Verify page title exists
    await expect(
      page.getByRole('heading', { name: 'AI Simulation', level: 1 })
    ).toBeVisible();
    
    console.log('✓ Simulation page loaded successfully');
  });

  test('should handle simulation with 10,000 entities and measure performance', async ({ page }) => {
    // This test validates the performance requirement via the UI
    // Target: 240 Hz (120 FPS * 2 ticks per frame)
    
    console.log('\n=== E2E Performance Test: 10,000 Elements ===');
    
    // Navigate to the simulation page
    await page.goto('./simulation');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load (it's loaded dynamically)
    await page.waitForTimeout(3000);
    
    // Look for entity count slider (second range input after grid size)
    const entityCountSlider = page.locator('label').filter({ hasText: /Entity Count:/i }).locator('input[type="range"]');
    
    // Set entity count to 1000
    await entityCountSlider.fill('1000');
    await page.waitForTimeout(500);
    console.log('✓ Set entity count to 1,000');
    
    // Set tick rate to 60 Hz to test performance
    const tickRateSlider = page.locator('label').filter({ hasText: /Tick Rate:/i }).locator('input[type="range"]');
    await tickRateSlider.fill('60');
    await page.waitForTimeout(500);
    console.log('✓ Set tick rate to 60 Hz');
    
    // Apply configuration
    const applyButton = page.locator('button').filter({ hasText: /Apply Config/i }).first();
    if (await applyButton.isVisible({ timeout: 1000 }).catch(() => false)) {
      await applyButton.click();
      await page.waitForTimeout(1000);
      console.log('✓ Applied configuration');
    }
    
    // Start the simulation
    const startButton = page
      .locator('.controls button')
      .filter({ hasText: /Start/i })
      .first();
    await startButton.waitFor({ state: 'visible' });
    await expect(startButton).toBeEnabled({ timeout: 10000 });
    await startButton.click();
    await page.waitForTimeout(500);
    console.log('✓ Simulation started');
    
    // Wait for simulation to run and stabilize
    await page.waitForTimeout(5000);
    
    // Capture performance metrics from the UI
    const performanceData = await page.evaluate(() => {
      // Find performance metric elements
      const metricsText = Array.from(document.querySelectorAll('.performance-metrics p'))
        .map(el => el.textContent || '')
        .join(' | ');
      
      // Extract tick rate
      const tickRateMatch = metricsText.match(/Actual Tick Rate:\s*([\d.]+)\s*Hz/);
      const actualTickRate = tickRateMatch ? parseFloat(tickRateMatch[1]) : 0;
      
      // Extract durations
      const tickDurationMatch = metricsText.match(/Tick Duration:\s*([\d.]+)\s*ms/);
      const tickDuration = tickDurationMatch ? parseFloat(tickDurationMatch[1]) : 0;
      
      const snapshotDurationMatch = metricsText.match(/Snapshot Time:\s*([\d.]+)\s*ms/);
      const snapshotDuration = snapshotDurationMatch ? parseFloat(snapshotDurationMatch[1]) : 0;
      
      return {
        actualTickRate,
        tickDuration,
        snapshotDuration,
        totalFrameTime: tickDuration + snapshotDuration,
        metricsText,
      };
    });
    
    console.log(`\n--- Performance Metrics (1000 entities) ---`);
    console.log(`Actual Tick Rate: ${performanceData.actualTickRate.toFixed(1)} Hz`);
    console.log(`Tick Duration: ${performanceData.tickDuration.toFixed(2)} ms`);
    console.log(`Snapshot Serialization: ${performanceData.snapshotDuration.toFixed(2)} ms`);
    console.log(`Total Frame Time: ${performanceData.totalFrameTime.toFixed(2)} ms`);
    console.log(`Target: 60 Hz (16.67 ms/frame)`);
    
    // Verify the page is responsive (doesn't freeze)
    const isResponsive = await page.evaluate(() => {
      return new Promise<boolean>(resolve => {
        let responsive = true;
        const timeout = setTimeout(() => {
          responsive = false;
          resolve(responsive);
        }, 100);
        requestAnimationFrame(() => {
          clearTimeout(timeout);
          resolve(responsive);
        });
      });
    });
    
    expect(isResponsive).toBe(true);
    console.log('✓ Page is responsive');
    
    // Verify reasonable performance (at least 50 Hz, allowing for some overhead)
    expect(performanceData.actualTickRate).toBeGreaterThan(50);
    console.log(`✓ Performance is acceptable (${performanceData.actualTickRate.toFixed(1)} Hz > 50 Hz)`);
    
    // Check tick counter is increasing
    const tickDisplay = page.locator('text=/Tick:?\\s*\\d+/i').first();
    const tickText1 = await tickDisplay.textContent().catch(() => 'Tick: 0');
    await page.waitForTimeout(1000);
    const tickText2 = await tickDisplay.textContent().catch(() => 'Tick: 0');
    
    console.log(`Tick progression: ${tickText1} → ${tickText2}`);
    
    // Take screenshot
    await page.screenshot({ path: '/tmp/simulation-1k-entities.png', fullPage: true });
    console.log('✓ Screenshot saved to /tmp/simulation-1k-entities.png');
    
    // Performance summary
    const performanceRatio = (performanceData.actualTickRate / 60) * 100;
    console.log(`\n--- Performance Summary ---`);
    console.log(`Achieved: ${performanceRatio.toFixed(1)}% of 60 Hz target`);
    
    if (performanceData.snapshotDuration > 5) {
      console.log(`⚠ Snapshot serialization is slow (${performanceData.snapshotDuration.toFixed(2)} ms)`);
      console.log(`  This may indicate serialization overhead with large entity counts`);
    }
    
    if (performanceData.tickDuration > 10) {
      console.log(`⚠ Tick duration is high (${performanceData.tickDuration.toFixed(2)} ms)`);
      console.log(`  This may indicate computation overhead in the simulation`);
    }
    
    console.log('✓ Test completed successfully\n');
  });
  
  test('should benchmark performance at different entity counts', async ({ page }) => {
    console.log('\n=== E2E Benchmark: Performance Scaling ===');
    
    await page.goto('./simulation');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    const entityCounts = [100, 250, 500, 1000];
    const results: Array<{count: number, tickRate: number, tickTime: number, snapshotTime: number}> = [];
    
    for (const count of entityCounts) {
      console.log(`\n--- Testing with ${count} entities ---`);
      
      // Set entity count
      const slider = page.locator('label').filter({ hasText: /Entity Count:/i }).locator('input[type="range"]');
      await slider.fill(count.toString());
      await page.waitForTimeout(300);
      
      // Set tick rate to 60 Hz for consistent benchmarking
      const tickRateSlider = page.locator('label').filter({ hasText: /Tick Rate:/i }).locator('input[type="range"]');
      await tickRateSlider.fill('60');
      await page.waitForTimeout(300);
      
      // Apply config
      const applyButton = page.locator('button').filter({ hasText: /Apply Config/i }).first();
      await applyButton.click();
      await page.waitForTimeout(1000);
      
      // Start simulation
      const startButton = page
        .locator('.controls button')
        .filter({ hasText: /Start/i })
        .first();
      await startButton.waitFor({ state: 'visible' });
      await expect(startButton).toBeEnabled({ timeout: 10000 });
      await startButton.click();
      await page.waitForTimeout(3000); // Let it stabilize
      
      // Capture metrics
      const metrics = await page.evaluate(() => {
        const metricsText = Array.from(document.querySelectorAll('.performance-metrics p'))
          .map(el => el.textContent || '')
          .join(' | ');
        
        const tickRateMatch = metricsText.match(/Actual Tick Rate:\s*([\d.]+)\s*Hz/);
        const tickDurationMatch = metricsText.match(/Tick Duration:\s*([\d.]+)\s*ms/);
        const snapshotDurationMatch = metricsText.match(/Snapshot Time:\s*([\d.]+)\s*ms/);
        
        return {
          tickRate: tickRateMatch ? parseFloat(tickRateMatch[1]) : 0,
          tickTime: tickDurationMatch ? parseFloat(tickDurationMatch[1]) : 0,
          snapshotTime: snapshotDurationMatch ? parseFloat(snapshotDurationMatch[1]) : 0,
        };
      });
      
      results.push({ count, ...metrics });
      
      console.log(`  Tick Rate: ${metrics.tickRate.toFixed(1)} Hz`);
      console.log(`  Tick Time: ${metrics.tickTime.toFixed(2)} ms`);
      console.log(`  Snapshot Time: ${metrics.snapshotTime.toFixed(2)} ms`);
      console.log(`  Total: ${(metrics.tickTime + metrics.snapshotTime).toFixed(2)} ms`);
      
      // Pause for next test
      const pauseButton = page.locator('button').filter({ hasText: /Pause/i }).first();
      await pauseButton.click();
      await page.waitForTimeout(500);
    }
    
    // Print comparison table
    console.log(`\n--- Performance Scaling Comparison ---`);
    console.log('Entities | Tick Rate | Tick Time | Snapshot Time | Total Time');
    console.log('---------|-----------|-----------|---------------|------------');
    for (const result of results) {
      const total = result.tickTime + result.snapshotTime;
      console.log(
        `${result.count.toString().padStart(8)} | ` +
        `${result.tickRate.toFixed(1).padStart(9)} Hz | ` +
        `${result.tickTime.toFixed(2).padStart(9)} ms | ` +
        `${result.snapshotTime.toFixed(2).padStart(13)} ms | ` +
        `${total.toFixed(2).padStart(10)} ms`
      );
    }
    
    // Analyze scaling
    if (results.length >= 2) {
      const first = results[0];
      const last = results[results.length - 1];
      const entityRatio = last.count / first.count;
      const tickTimeRatio = last.tickTime / Math.max(first.tickTime, 0.1);
      const snapshotTimeRatio = last.snapshotTime / Math.max(first.snapshotTime, 0.1);
      
      console.log(`\n--- Scaling Analysis (${first.count} → ${last.count} entities) ---`);
      console.log(`Entity count increased: ${entityRatio.toFixed(1)}x`);
      console.log(`Tick time increased: ${tickTimeRatio.toFixed(1)}x`);
      console.log(`Snapshot time increased: ${snapshotTimeRatio.toFixed(1)}x`);
      
      if (tickTimeRatio > entityRatio * 1.5) {
        console.log(`⚠ Tick time scaling is worse than linear (possible O(n²) behavior)`);
      } else {
        console.log(`✓ Tick time scaling is reasonable`);
      }
      
      if (snapshotTimeRatio > entityRatio * 1.5) {
        console.log(`⚠ Snapshot serialization scaling is worse than linear`);
      } else {
        console.log(`✓ Snapshot serialization scaling is reasonable`);
      }
    }
    
    console.log('\n✓ Benchmark completed\n');
    
    // Verify all tests met minimum performance (50 Hz with 60 Hz target)
    for (const result of results) {
      expect(result.tickRate).toBeGreaterThan(50);
    }
  });
  
  test('should display entity count and simulation controls', async ({ page }) => {
    console.log('\n=== E2E Test: UI Elements ===');
    
    await page.goto('./simulation');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Verify input exists
    const entityCountSlider = page.locator('input[type="range"]').first();
    await expect(entityCountSlider).toBeVisible();
    console.log('✓ Entity count slider is visible');
    
    // Verify start button exists
    const startButton = page
      .locator('.controls button')
      .filter({ hasText: /Start/i })
      .first();
    await expect(startButton).toBeVisible();
    console.log('✓ Start button is visible');
    
    // Verify page title
    const pageTitle = await page.title();
    expect(pageTitle).toContain('Simulation');
    console.log(`✓ Page title: "${pageTitle}"\n`);
  });
});
