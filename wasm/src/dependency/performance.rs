#[cfg(target_arch = "wasm32")]
thread_local! {
    static PERFORMANCE: Option<web_sys::Performance> =
        web_sys::window().and_then(|w| w.performance());
}

#[cfg(target_arch = "wasm32")]
pub fn performance_now() -> f64 {
    PERFORMANCE.with(|perf| perf.as_ref().map(|p| p.now()).unwrap_or(0.0))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn performance_now() -> f64 {
    0.0
}
