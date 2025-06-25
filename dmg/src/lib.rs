#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]
pub mod fde;
pub mod gb;
pub mod memory;
pub mod single_step_tests;
pub mod util;
mod log;
