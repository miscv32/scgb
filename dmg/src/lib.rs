#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]
pub mod fde;
pub mod gb;
mod log;
pub mod memory;
pub mod single_step_tests;
pub mod util;
pub mod renderer;
pub mod isr;
