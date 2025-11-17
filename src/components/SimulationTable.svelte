<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  
  // Types for AI entities
  interface AiEntity {
    id: number;
    health: number;
    military_strength: number;
    position_x: number;
    position_y: number;
    state: number | string;
    territory: number;
    money: number;
  }

  // State names for display
  const stateNames = ['Idle', 'Active', 'Resting', 'Moving', 'Dead'];

  // Helper function to get state name
  function getStateName(state: number | string): string {
    if (typeof state === 'string') {
      return state;
    }
    return stateNames[state] || 'Unknown';
  }

  // Simulation control state
  let simulation: any = null;
  let wasmLoaded: boolean = false;
  let error: string | null = null;
  
  // Simulation data
  let entities: AiEntity[] = [];
  let tick: number = 0;
  let isRunning: boolean = false;
  
  // Configuration - optimized for all devices (mobile as baseline)
  let entityCount: number = 250;
  let tickRate: number = 30;
  let renderRate: number = 30; // Manual control of render FPS
  
  // Performance metrics
  let actualTickRate: number = 0;
  let actualRenderRate: number = 0;
  let tickDuration: number = 0;
  let snapshotDuration: number = 0;
  let avgTickDuration: number = 0;
  let avgSnapshotDuration: number = 0;
  let durationSampleCount: number = 0;
  let lastTickTime: number = 0;
  let lastTickCount: number = 0;
  let tickRateMeasureTime: number = 0;
  let renderRateMeasureTime: number = 0;
  let renderFrameCount: number = 0;
  
  // Update loop
  let updateInterval: number | null = null;
  let renderInterval: number | null = null;
  
  // Filter state
  let hideDeadAIs: boolean = false;

  // Virtual scrolling for performance
  let visibleStartIndex: number = 0;
  let visibleEndIndex: number = 50; // Show 50 rows at a time
  let tableScrollTop: number = 0;
  const rowHeight: number = 45; // Approximate row height in pixels
  
  function handleTableScroll(event: Event): void {
    const target = event.target as HTMLElement;
    tableScrollTop = target.scrollTop;
    
    // Calculate visible range with buffer
    const buffer = 10; // Extra rows above/below for smooth scrolling
    visibleStartIndex = Math.max(0, Math.floor(tableScrollTop / rowHeight) - buffer);
    visibleEndIndex = Math.min(
      filteredEntities.length,
      Math.ceil((tableScrollTop + target.clientHeight) / rowHeight) + buffer
    );
  }

  // Load the WASM module on component mount
  onMount(async () => {
    try {
      // Dynamically import the WASM module
      const wasmModule = await import('../wasm/wasm.js');
      await wasmModule.default();
      
      // Initialize the simulation
      simulation = new wasmModule.Simulation(entityCount);
      wasmLoaded = true;
      
      // Get initial snapshot
      updateSnapshot();
      
      // Start render loop (independent of simulation tick rate)
      startRenderLoop();
    } catch (e) {
      console.error('Failed to load WASM module:', e);
      error = 'Failed to load WebAssembly module';
      wasmLoaded = false;
    }
  });

  onDestroy(() => {
    stopSimulation();
    stopRenderLoop();
    if (simulation) {
      simulation.destroy();
    }
  });

  function startSimulation(): void {
    if (!simulation || !wasmLoaded) return;
    
    simulation.start();
    isRunning = true;
    
    // Update loop runs at tick rate
    const interval = 1000 / tickRate;
    updateInterval = window.setInterval(() => {
      simulation.update();
    }, interval);
  }

  function pauseSimulation(): void {
    if (!simulation || !wasmLoaded) return;
    
    simulation.pause();
    isRunning = false;
    
    if (updateInterval !== null) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
  }

  function resumeSimulation(): void {
    if (!simulation || !wasmLoaded) return;
    
    if (isRunning) return; // Already running
    
    simulation.resume();
    isRunning = true;
    
    const interval = 1000 / tickRate;
    updateInterval = window.setInterval(() => {
      simulation.update();
    }, interval);
  }

  function stepSimulation(): void {
    if (!simulation || !wasmLoaded) return;
    simulation.step();
    updateSnapshot();
  }

  function resetSimulation(): void {
    if (!simulation || !wasmLoaded) return;
    
    pauseSimulation();
    simulation.reset();
    updateSnapshot();
  }

  function applyConfiguration(): void {
    if (!simulation || !wasmLoaded) return;
    
    pauseSimulation();
    simulation.set_entity_count(entityCount);
    simulation.set_tick_rate(tickRate);
    updateSnapshot();
    
    // Reset averages when config changes
    avgTickDuration = 0;
    avgSnapshotDuration = 0;
    durationSampleCount = 0;
    
    // Update render loop to match new render rate
    updateRenderLoop();
  }

  function applyPerformanceMode(): void {
    if (!simulation || !wasmLoaded) return;
    
    // Performance presets (same for all devices)
    const presets = {
      low: { entities: 100, tickRate: 30 },
      medium: { entities: 250, tickRate: 30 },
      high: { entities: 500, tickRate: 30 },
    };
    
    const preset = presets[performanceMode];
    entityCount = preset.entities;
    tickRate = preset.tickRate;
    
    applyConfiguration();
  }

  function startRenderLoop(): void {
    // Use configured render rate
    const renderInterval_ms = 1000 / renderRate;
    
    renderInterval = window.setInterval(() => {
      updateSnapshot();
    }, renderInterval_ms);
  }

  function stopRenderLoop(): void {
    if (renderInterval !== null) {
      clearInterval(renderInterval);
      renderInterval = null;
    }
  }

  function updateRenderLoop(): void {
    // Restart render loop with new rate when tick rate changes
    stopRenderLoop();
    if (wasmLoaded) {
      startRenderLoop();
    }
  }

  function stopSimulation(): void {
    if (updateInterval !== null) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
    isRunning = false;
  }

  function updateSnapshot(): void {
    if (!simulation || !wasmLoaded) return;
    
    const currentTime = performance.now();
    const currentTick = Number(simulation.get_tick()); // Convert BigInt to number
    
    // Track render frame count
    renderFrameCount++;
    
    // Calculate actual render rate
    if (renderRateMeasureTime > 0) {
      const timeDelta = currentTime - renderRateMeasureTime;
      if (timeDelta >= 1000) { // Update every second
        actualRenderRate = (renderFrameCount / timeDelta) * 1000;
        renderRateMeasureTime = currentTime;
        renderFrameCount = 0;
      }
    } else {
      renderRateMeasureTime = currentTime;
    }
    
    // Calculate actual tick rate based on tick count changes over time
    if (tickRateMeasureTime > 0) {
      const timeDelta = currentTime - tickRateMeasureTime;
      const tickDelta = currentTick - lastTickCount;
      
      if (timeDelta >= 1000) { // Update every second
        actualTickRate = (tickDelta / timeDelta) * 1000;
        tickRateMeasureTime = currentTime;
        lastTickCount = currentTick;
      }
    } else {
      tickRateMeasureTime = currentTime;
      lastTickCount = currentTick;
    }
    
    tick = currentTick;
    tickDuration = simulation.get_last_tick_duration();
    snapshotDuration = simulation.get_last_snapshot_duration();
    
    // Calculate running averages for durations (more stable/readable)
    durationSampleCount++;
    if (tickDuration > 0) {
      avgTickDuration = avgTickDuration + (tickDuration - avgTickDuration) / Math.min(durationSampleCount, 100);
    }
    if (snapshotDuration > 0) {
      avgSnapshotDuration = avgSnapshotDuration + (snapshotDuration - avgSnapshotDuration) / Math.min(durationSampleCount, 100);
    }
    
    const snapshot = simulation.get_snapshot();
    
    // Only update entities if snapshot is not null (data changed)
    if (snapshot !== null && snapshot !== undefined) {
      entities = snapshot || [];
    }
  }

  function formatNumber(num: number): string {
    return num.toFixed(2);
  }
  
  // Filtered entities based on hideDeadAIs state
  $: filteredEntities = hideDeadAIs 
    ? entities.filter(entity => getStateName(entity.state) !== 'Dead')
    : entities;
  
  // Visible entities for virtual scrolling (only render what's on screen)
  $: visibleEntities = filteredEntities.slice(visibleStartIndex, visibleEndIndex);
  
  // Calculate padding for virtual scroll
  $: topPadding = visibleStartIndex * rowHeight;
  $: bottomPadding = Math.max(0, (filteredEntities.length - visibleEndIndex) * rowHeight);
</script>

<div class="simulation-container">
  <div class="header">
    <h2>AI Simulation</h2>
    {#if error}
      <p class="error">{error}</p>
    {/if}
    {#if wasmLoaded}
      <p class="wasm-badge">⚡ Powered by Rust + WebAssembly</p>
    {/if}
  </div>

  <div class="controls">
    <div class="control-group">
      <h3>Simulation Controls</h3>
      <div class="button-group">
        <button 
          on:click={startSimulation} 
          class="btn btn-success"
          disabled={!wasmLoaded || isRunning}
          aria-label="Start simulation"
        >
          ▶ Start
        </button>
        
        <button 
          on:click={pauseSimulation} 
          class="btn btn-warning"
          disabled={!wasmLoaded || !isRunning}
          aria-label="Pause simulation"
        >
          ⏸ Pause
        </button>
        
        <button 
          on:click={resumeSimulation} 
          class="btn btn-success"
          disabled={!wasmLoaded || isRunning}
          aria-label="Resume simulation"
        >
          ▶ Resume
        </button>
        
        <button 
          on:click={stepSimulation} 
          class="btn btn-primary"
          disabled={!wasmLoaded}
          aria-label="Step simulation"
        >
          ⏭ Step
        </button>
        
        <button 
          on:click={resetSimulation} 
          class="btn btn-danger"
          disabled={!wasmLoaded}
          aria-label="Reset simulation"
        >
          ⏹ Reset
        </button>
      </div>
    </div>

    <div class="control-group">
      <h3>Configuration</h3>
      <div class="config-inputs">
        <label>
          Entity Count: <span>{entityCount}</span>
          <input 
            type="range" 
            min="10" 
            max="1000" 
            step="10"
            bind:value={entityCount}
          />
        </label>
        
        <label>
          Tick Rate: <span>{tickRate} Hz</span>
          <input 
            type="range" 
            min="1" 
            max="120" 
            step="1"
            bind:value={tickRate}
          />
        </label>
        
        <label>
          Render Rate: <span>{renderRate} FPS</span>
          <input 
            type="range" 
            min="1" 
            max="60" 
            step="1"
            bind:value={renderRate}
            on:change={updateRenderLoop}
          />
        </label>
        
        <button 
          on:click={applyConfiguration} 
          class="btn btn-primary"
          disabled={!wasmLoaded}
        >
          Apply Config
        </button>
      </div>
    </div>

    <div class="stats">
      <p><strong>Tick:</strong> {tick}</p>
      <p><strong>Entities:</strong> {filteredEntities.length}{#if hideDeadAIs} / {entities.length}{/if}</p>
      <p><strong>Status:</strong> {isRunning ? '🟢 Running' : '🔴 Paused'}</p>
      <div class="performance-metrics">
        <p><strong>Target Tick Rate:</strong> {tickRate} Hz</p>
        <p><strong>Actual Tick Rate:</strong> {actualTickRate.toFixed(1)} Hz</p>
        <p><strong>Target Render Rate:</strong> {renderRate} FPS</p>
        <p><strong>Actual Render Rate:</strong> {actualRenderRate.toFixed(1)} FPS</p>
        <p><strong>Avg Tick Duration:</strong> {avgTickDuration.toFixed(2)} ms</p>
        <p><strong>Avg Snapshot Time:</strong> {avgSnapshotDuration.toFixed(2)} ms</p>
        <p class="perf-indicator">
          <strong>Performance:</strong> 
          {#if actualTickRate >= tickRate * 0.9}
            <span class="perf-good">✓ Good</span>
          {:else if actualTickRate >= tickRate * 0.5}
            <span class="perf-warn">⚠ Degraded</span>
          {:else}
            <span class="perf-bad">✗ Poor</span>
          {/if}
        </p>
      </div>
      <label class="filter-checkbox">
        <input 
          type="checkbox" 
          bind:checked={hideDeadAIs}
        />
        <span>Hide Dead AIs</span>
      </label>
    </div>
  </div>

  <div class="table-container" on:scroll={handleTableScroll}>
    <table class="entity-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Health</th>
          <th>Military Strength</th>
          <th>Money</th>
          <th>Territory</th>
          <th>Position X</th>
          <th>Position Y</th>
          <th>State</th>
        </tr>
      </thead>
      <tbody>
        <!-- Spacer for virtual scroll top padding -->
        {#if topPadding > 0}
          <tr style="height: {topPadding}px; pointer-events: none;">
            <td colspan="8"></td>
          </tr>
        {/if}
        
        {#each visibleEntities as entity (entity.id)}
          <tr>
            <td>{entity.id}</td>
            <td class="health-cell">
              <div class="bar-container">
                <div class="bar bar-health" style="width: {entity.health}%"></div>
                <span class="bar-text">{formatNumber(entity.health)}</span>
              </div>
            </td>
            <td class="military-cell">
              <div class="bar-container">
                <div class="bar bar-military" style="width: {entity.military_strength}%"></div>
                <span class="bar-text">{formatNumber(entity.military_strength)}</span>
              </div>
            </td>
            <td class="money-cell">
              <div class="bar-container">
                <div class="bar bar-money" style="width: {Math.min(entity.money / 2, 100)}%"></div>
                <span class="bar-text">{formatNumber(entity.money)}</span>
              </div>
            </td>
            <td class="territory-cell">
              <div class="bar-container">
                <div class="bar bar-territory" style="width: {entity.territory}%"></div>
                <span class="bar-text">{formatNumber(entity.territory)}</span>
              </div>
            </td>
            <td>{formatNumber(entity.position_x)}</td>
            <td>{formatNumber(entity.position_y)}</td>
            <td class="state-cell">
              <span class="state-badge state-{typeof entity.state === 'number' ? entity.state : 0}">
                {getStateName(entity.state)}
              </span>
            </td>
          </tr>
        {/each}
        
        <!-- Spacer for virtual scroll bottom padding -->
        {#if bottomPadding > 0}
          <tr style="height: {bottomPadding}px; pointer-events: none;">
            <td colspan="8"></td>
          </tr>
        {/if}
      </tbody>
    </table>
  </div>
</div>

<style>
  .simulation-container {
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
  }

  .header {
    text-align: center;
    margin-bottom: 1.5rem;
  }

  .header h2 {
    margin: 0 0 0.5rem 0;
    color: #333;
  }

  .wasm-badge {
    font-size: 0.875rem;
    color: #10b981;
    font-weight: 600;
    margin: 0.5rem 0;
  }

  .error {
    color: #ef4444;
    font-size: 0.875rem;
    margin: 0.5rem 0;
  }

  .controls {
    background: #f5f5f5;
    border-radius: 0.5rem;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .control-group {
    margin-bottom: 1rem;
  }

  .control-group:last-child {
    margin-bottom: 0;
  }

  .control-group h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    color: #555;
  }

  .button-group {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .config-inputs {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .config-inputs label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.875rem;
    color: #555;
  }

  .config-inputs label span {
    font-weight: 600;
    color: #333;
  }

  .config-inputs input[type="range"] {
    width: 100%;
  }

  .stats {
    display: flex;
    gap: 2rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #ddd;
    flex-wrap: wrap;
  }

  .stats p {
    margin: 0;
    font-size: 0.875rem;
    color: #555;
  }

  .performance-metrics {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.5rem;
    background: #f9fafb;
    border-radius: 0.375rem;
    border: 1px solid #e5e7eb;
  }

  .performance-metrics p {
    margin: 0;
    font-size: 0.75rem;
  }

  .perf-indicator {
    font-weight: 600;
  }

  .perf-good {
    color: #10b981;
  }

  .perf-warn {
    color: #f59e0b;
  }

  .perf-bad {
    color: #ef4444;
  }

  .filter-checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: #555;
    cursor: pointer;
    user-select: none;
  }

  .filter-checkbox input[type="checkbox"] {
    cursor: pointer;
    width: 16px;
    height: 16px;
  }

  .filter-checkbox span {
    font-weight: 500;
  }

  .btn {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: white;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .btn:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.15);
  }

  .btn:active:not(:disabled) {
    transform: scale(0.98);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #667eea;
  }

  .btn-primary:hover:not(:disabled) {
    background: #5568d3;
  }

  .btn-success {
    background: #10b981;
  }

  .btn-success:hover:not(:disabled) {
    background: #059669;
  }

  .btn-warning {
    background: #f59e0b;
  }

  .btn-warning:hover:not(:disabled) {
    background: #d97706;
  }

  .btn-danger {
    background: #ef4444;
  }

  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
  }

  .table-container {
    overflow: auto;
    max-height: 600px;
    border: 1px solid #ddd;
    border-radius: 0.5rem;
    background: white;
  }

  .entity-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  .entity-table thead {
    position: sticky;
    top: 0;
    background: #667eea;
    color: white;
    z-index: 10;
  }

  .entity-table th {
    padding: 0.75rem;
    text-align: left;
    font-weight: 600;
    border-bottom: 2px solid #5568d3;
  }

  .entity-table tbody tr {
    border-bottom: 1px solid #e5e7eb;
  }

  .entity-table tbody tr:hover {
    background-color: #f9fafb;
  }

  .entity-table td {
    padding: 0.5rem 0.75rem;
  }

  .bar-container {
    position: relative;
    width: 100%;
    height: 20px;
    background: #e5e7eb;
    border-radius: 0.25rem;
    overflow: hidden;
  }

  .bar {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    border-radius: 0.25rem;
    /* No transitions for consistent performance across all devices */
  }

  .bar-health {
    background: linear-gradient(90deg, #ef4444 0%, #10b981 100%);
  }

  .bar-military {
    background: linear-gradient(90deg, #6366f1 0%, #8b5cf6 100%);
  }

  .bar-territory {
    background: linear-gradient(90deg, #eab308 0%, #22c55e 100%);
  }

  .bar-money {
    background: linear-gradient(90deg, #f59e0b 0%, #10b981 100%);
  }

  .bar-text {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 0.75rem;
    font-weight: 600;
    color: #333;
    z-index: 1;
    text-shadow: 0 0 2px white;
  }

  .state-badge {
    display: inline-block;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-align: center;
  }

  .state-0 {
    background: #e5e7eb;
    color: #6b7280;
  }

  .state-1 {
    background: #dbeafe;
    color: #1e40af;
  }

  .state-2 {
    background: #fef3c7;
    color: #92400e;
  }

  .state-3 {
    background: #d1fae5;
    color: #065f46;
  }

  .state-4 {
    background: #fee2e2;
    color: #991b1b;
  }

  @media (max-width: 768px) {
    .button-group {
      flex-direction: column;
    }

    .btn {
      width: 100%;
    }

    .stats {
      flex-direction: column;
      gap: 0.5rem;
    }

    .table-container {
      max-height: 400px;
    }
  }
</style>
