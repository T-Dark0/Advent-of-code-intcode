use std::{collections::HashMap, convert::TryFrom};

use derive_more::{Display, IntoIterator};

#[derive(Debug, Display, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct Value(pub i32);

#[derive(Debug, Display, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Address(pub u32);

impl TryFrom<Value> for Address {
    type Error = TryFromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match u32::try_from(value.0) {
            Ok(val) => Ok(Address(val)),
            Err(_) => Err(TryFromValueError::OutOfRange(value)),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, IntoIterator)]
pub struct Memory(HashMap<Address, Value>);

impl Memory {
    pub fn new(memory: HashMap<Address, Value>) -> Self {
        Memory(memory)
    }

    pub fn read(&self, addr: Address) -> Result<Value, Error> {
        self.0.get(&addr).copied().ok_or(Error::EmptyRead(addr))
    }

    pub fn write(&mut self, addr: Address, val: Value) {
        self.0.insert(addr, val);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    EmptyRead(Address),
}

#[derive(Debug, Eq, PartialEq)]
pub enum TryFromValueError {
    OutOfRange(Value),
}
