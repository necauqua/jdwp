#![deny(missing_debug_implementations, clippy::undocumented_unsafe_blocks)]

extern crate self as jdwp;

pub mod client;
pub mod codec;
pub mod highlevel;
pub mod jvm;
pub mod spec;

mod functional;
mod xorshift;

pub(crate) use jdwp_macros::jdwp_command;
