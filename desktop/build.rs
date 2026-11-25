//! On Windows laptops with two GPUs, ask for the discrete one.
//!
//! gpui renders on the first adapter DXGI lists, which on a hybrid laptop is
//! the integrated GPU. NVIDIA (Optimus) and AMD (PowerXpress) drivers look
//! for these two symbols in the executable's export table and run the app on
//! the discrete GPU when they're set; the statics are in `spec01::gpu`.
//! Executables don't export symbols by default, so the linker is told to.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("windows") && target.contains("msvc") {
        println!("cargo:rustc-link-arg-bins=/EXPORT:NvOptimusEnablement");
        println!("cargo:rustc-link-arg-bins=/EXPORT:AmdPowerXpressRequestHighPerformance");
    }
}
