use cfg_if;

#[cfg(not(feature = "check-loom"))]
pub use std::*;

#[cfg(feature = "check-loom")]
pub use loom::*;

/// Run `f` with `loom::model` if compiled with `check-loom` feature.
pub fn model<F: Fn() + Sync + Send + 'static>(f: F) {
    cfg_if::cfg_if! {
        if #[cfg(feature = "check-loom")] {
            loom::model(f)
        } else {
            f()
        }
    }
}
