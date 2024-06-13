// pub mod emulator;
// pub use emulator::{input::InputBuilder, machine::Machine, output::*};

pub mod machine;
pub mod test_runner;

pub use machine::Machine;
pub use test_runner::*;
