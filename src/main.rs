mod app;

pub use app::make_start;
extern crate eframe;
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eframe::run_native(Box::new(make_start()), Default::default())
}