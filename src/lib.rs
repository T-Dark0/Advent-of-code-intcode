#![feature(min_const_generics)]
#![feature(array_map)]
#![feature(iter_map_while)]

pub mod memory;
pub mod processor;

pub use memory::{Address, Error as MemoryError, Memory, Value};
pub use processor::{Error as ProcessorError, Processor};
