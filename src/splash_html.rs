//! Splash screen hooks for the WASM HTML shell (`mq_set_progress` / `mq_hide_splash`).

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn mq_set_progress(percent: f32);
    fn mq_hide_splash();
}

/// Report loading progress (0–100) to the HTML splash screen.
pub fn set_loading_progress(percent: f32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        mq_set_progress(percent);
    }
}

/// Hide the splash overlay once assets are ready.
pub fn hide_loading_splash() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        mq_hide_splash();
    }
}
