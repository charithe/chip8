mod common;
pub mod debugger;
mod display;
mod implementation;
mod interpreter;
pub mod ui;

pub type Error = common::Error;
pub type Result<T> = common::Result<T>;
pub type Emulator = implementation::Emulator;
pub type Input = implementation::Input;
pub type Step = implementation::Step;
