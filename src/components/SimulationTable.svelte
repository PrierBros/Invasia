<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  
  // Types for AI entities
  interface AiEntity {
    id: number;
    health: number;
    energy: number;
    position_x: number;
    position_y: number;
    state: number | string;
  }

  // State names for display
  const stateNames = ['Idle', 'Active', 'Resting', 'Moving'];

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
  
  // Configuration
  let entityCount: number = 100;
  let tickRate: number = 60;
  
  // Update loop
  let updateInterval: number | null = null;
  let renderInterval: number | null = null;

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
  }

  function startRenderLoop(): void {
    // Render at ~30 FPS (independent of simulation tick rate)
    renderInterval = window.setInterval(() => {
      updateSnapshot();
    }, 1000 / 30);
  }

  function stopRenderLoop(): void {
    if (renderInterval !== null) {
      clearInterval(renderInterval);
      renderInterval = null;
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
    
    tick = simulation.get_tick();
    const snapshot = simulation.get_snapshot();
    entities = snapshot || [];
  }

  function formatNumber(num: number): string {
    return num.toFixed(2);
  }
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
      <p><strong>Entities:</strong> {entities.length}</p>
      <p><strong>Status:</strong> {isRunning ? '🟢 Running' : '🔴 Paused'}</p>
    </div>
  </div>

  <div class="table-container">
    <table class="entity-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Health</th>
          <th>Energy</th>
          <th>Position X</th>
          <th>Position Y</th>
          <th>State</th>
        </tr>
      </thead>
      <tbody>
        {#each entities as entity (entity.id)}
          <tr>
            <td>{entity.id}</td>
            <td class="health-cell">
              <div class="bar-container">
                <div class="bar bar-health" style="width: {entity.health}%"></div>
                <span class="bar-text">{formatNumber(entity.health)}</span>
              </div>
            </td>
            <td class="energy-cell">
              <div class="bar-container">
                <div class="bar bar-energy" style="width: {entity.energy}%"></div>
                <span class="bar-text">{formatNumber(entity.energy)}</span>
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
  }

  .stats p {
    margin: 0;
    font-size: 0.875rem;
    color: #555;
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
    transition: width 0.3s ease;
  }

  .bar-health {
    background: linear-gradient(90deg, #ef4444 0%, #10b981 100%);
  }

  .bar-energy {
    background: linear-gradient(90deg, #f59e0b 0%, #3b82f6 100%);
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
