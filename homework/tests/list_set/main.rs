// optimistic_fine_grained on thread santizer has very unstable performance on gg.
#![feature(cfg_sanitize)]

mod fine_grained;
mod optimistic_fine_grained;
