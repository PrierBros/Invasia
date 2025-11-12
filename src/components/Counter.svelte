<script lang="ts">
  import { onMount } from 'svelte';
  
  // Counter state managed by Svelte
  let count: number = 0;
  let counter: any = null;
  let wasmLoaded: boolean = false;
  let error: string | null = null;

  // Load the WASM module on component mount
  onMount(async () => {
    try {
      // Dynamically import the WASM module
      const wasmModule = await import('../wasm/wasm_counter.js');
      await wasmModule.default();
      
      // Initialize the counter with Rust/WASM
      counter = new wasmModule.Counter();
      count = counter.get_value();
      wasmLoaded = true;
    } catch (e) {
      console.error('Failed to load WASM module:', e);
      error = 'Failed to load WebAssembly module';
      wasmLoaded = false;
    }
  });

  // Increment handler using WASM logic
  function handleIncrement(): void {
    if (counter && wasmLoaded) {
      count = counter.increment();
    } else {
      // Fallback to JavaScript if WASM not loaded
      count++;
    }
  }

  // Decrement handler using WASM logic
  function handleDecrement(): void {
    if (counter && wasmLoaded) {
      count = counter.decrement();
    } else {
      // Fallback to JavaScript if WASM not loaded
      count--;
    }
  }

  // Reset handler using WASM logic
  function handleReset(): void {
    if (counter && wasmLoaded) {
      counter.reset();
      count = counter.get_value();
    } else {
      count = 0;
    }
  }
</script>

<div class="counter-container">
  <div class="counter">
    <p class="count-label">
      Count: <span class="count-value">{count}</span>
    </p>
    
    {#if error}
      <p class="error">{error}</p>
    {/if}
    
    {#if wasmLoaded}
      <p class="wasm-badge">⚡ Powered by Rust + WebAssembly</p>
    {/if}
    
    <div class="button-group">
      <button 
        on:click={handleDecrement} 
        class="btn btn-decrement"
        aria-label="Decrement counter"
      >
        Decrement
      </button>
      
      <button 
        on:click={handleReset} 
        class="btn btn-reset"
        aria-label="Reset counter"
      >
        Reset
      </button>
      
      <button 
        on:click={handleIncrement} 
        class="btn btn-increment"
        aria-label="Increment counter"
      >
        Increment
      </button>
    </div>
  </div>
</div>

<style>
  .counter-container {
    width: 100%;
  }

  .counter {
    padding: 1.5rem;
    background: #f5f5f5;
    border-radius: 0.5rem;
    margin-top: 1rem;
  }

  .count-label {
    font-size: 1.5rem;
    font-weight: bold;
    margin-bottom: 1rem;
    color: #333;
  }

  .count-value {
    color: #667eea;
    font-size: 2rem;
  }

  .wasm-badge {
    font-size: 0.875rem;
    color: #10b981;
    font-weight: 600;
    margin: 0.5rem 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
  }

  .error {
    color: #ef4444;
    font-size: 0.875rem;
    margin: 0.5rem 0;
  }

  .button-group {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    flex-wrap: wrap;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 500;
    color: white;
    border: none;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
  }

  .btn:active {
    transform: scale(0.95);
  }

  .btn-increment {
    background: #667eea;
  }

  .btn-increment:hover {
    background: #5568d3;
  }

  .btn-decrement {
    background: #f59e0b;
  }

  .btn-decrement:hover {
    background: #d97706;
  }

  .btn-reset {
    background: #64748b;
  }

  .btn-reset:hover {
    background: #475569;
  }

  @media (max-width: 640px) {
    .button-group {
      flex-direction: column;
    }

    .btn {
      width: 100%;
    }
  }
</style>
