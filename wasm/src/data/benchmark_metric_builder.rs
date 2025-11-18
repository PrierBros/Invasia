pub struct BenchmarkMetricBuilder;

impl BenchmarkMetricBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn measure_tick<F, T>(&self, mut f: F) -> (T, f64)
    where
        F: FnMut() -> T,
    {
        let start = performance_now();
        let result = f();
        let duration = elapsed_duration(start);
        (result, duration)
    }

    pub fn measure_snapshot<F, T>(&self, mut f: F) -> (T, f64)
    where
        F: FnMut() -> T,
    {
        let start = performance_now();
        let result = f();
        let duration = elapsed_duration(start);
        (result, duration)
    }
}

fn elapsed_duration(start: f64) -> f64 {
    if start <= 0.0 {
        return 0.0;
    }
    let end = performance_now();
    if end >= start {
        end - start
    } else {
        0.0
    }
}

#[cfg(target_arch = "wasm32")]
fn performance_now() -> f64 {
    thread_local! {
        static PERFORMANCE: Option<web_sys::Performance> =
            web_sys::window().and_then(|w| w.performance());
    }

    PERFORMANCE.with(|perf| perf.as_ref().map(|p| p.now()).unwrap_or(0.0))
}

#[cfg(not(target_arch = "wasm32"))]
fn performance_now() -> f64 {
    0.0
}
