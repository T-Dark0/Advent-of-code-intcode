mod handlers;

use std::{collections::VecDeque, convert::TryFrom};

use crate::memory::{self, Address, Memory, Value};

use derive_more::From;

#[derive(Debug, Clone)]
pub struct Processor {
    pc: Address,
    relative_base: Value,
    memory: Memory,
    input_buffer: VecDeque<Value>,
}

impl Processor {
    pub fn new(memory: Memory) -> Self {
        Processor {
            pc: Address(0),
            relative_base: Value(0),
            memory,
            input_buffer: VecDeque::new(),
        }
    }

    pub fn execute_once(&mut self) -> ProcessorState {
        let modes_and_opcode = self.memory.read(self.pc);

        let modes = modes_and_opcode.0 / 100;
        let opcode = Value(modes_and_opcode.0 % 100);

        let result = match opcode.0 {
            1 => self.add(modes),
            2 => self.multiply(modes),
            3 => self.input(modes),
            4 => self.output(modes),
            5 => self.jump_if_true(modes),
            6 => self.jump_if_false(modes),
            7 => self.less_than(modes),
            8 => self.equals(modes),
            9 => self.adjust_relative_base(modes),
            0 => return ProcessorState::Error(Error::FinishedWithoutTerminating),
            99 => return ProcessorState::Terminate,
            _ => return ProcessorState::Error(Error::InvalidOpcode),
        };
        match result {
            Ok(opt_out) => ProcessorState::Continue(opt_out),
            Err(err) => return ProcessorState::Error(err),
        }
    }

    pub fn execute(&mut self) -> Result<(), Error> {
        loop {
            match self.execute_once() {
                ProcessorState::Continue(_) => continue,
                ProcessorState::Terminate => break Ok(()),
                ProcessorState::Error(err) => break Err(err),
            }
        }
    }

    pub fn execute_until_output(&mut self) -> Option<Result<Value, Error>> {
        loop {
            match self.execute_once() {
                ProcessorState::Continue(opt_out) => match opt_out {
                    Some(out) => break Some(Ok(out)),
                    None => continue,
                },
                ProcessorState::Terminate => break None,
                ProcessorState::Error(err) => break Some(Err(err)),
            }
        }
    }

    pub fn push_input(&mut self, value: Value) {
        self.input_buffer.push_back(value)
    }
}

pub enum ProcessorState {
    Continue(Option<Value>),
    Terminate,
    Error(Error),
}

#[derive(Debug, From, Eq, PartialEq)]
pub enum Error {
    IllegalPositionalArgument(memory::TryFromValueError),
    IllegalMode,
    InputReadError,
    InvalidOpcode,
    FinishedWithoutTerminating,
}

type Modes = i64;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Mode {
    Positional,
    Immediate,
    Relative,
}

impl TryFrom<Modes> for Mode {
    type Error = ModeTryFromError;

    fn try_from(value: Modes) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Mode::Positional),
            1 => Ok(Mode::Immediate),
            2 => Ok(Mode::Relative),
            _ => Err(ModeTryFromError::InvalidMode),
        }
    }
}

pub enum ModeTryFromError {
    InvalidMode,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::{Address, Value};
    use maplit::hashmap;

    impl Processor {
        fn get_memory(&self) -> &Memory {
            &self.memory
        }
    }

    macro_rules! memory {
        ($($key:expr => $value:expr,)+) => { memory!($($key => $value),+) };
        ($($key:expr => $value:expr),*) => {
            hashmap! {
                $(
                    Address($key) => Value($value),
                )*
            };
        };
    }

    #[test]
    fn add() {
        let memory = Memory::new(memory! {
            0 => 1,
            1 => 10,
            2 => 20,
            3 => 100,
            4 => 99,

            10 => 41,
            20 => 9,
        });
        let mut processor = Processor::new(memory);
        processor.execute().unwrap();

        let expected = Memory::new(memory! {
            0 => 1,
            1 => 10,
            2 => 20,
            3 => 100,
            4 => 99,

            10 => 41,
            20 => 9,
            100 => 41 + 9,
        });

        assert_eq!(processor.get_memory(), &expected);
    }

    #[test]
    fn multiply() {
        let memory = Memory::new(memory! {
            0 => 2,
            1 => 10,
            2 => 20,
            3 => 100,
            4 => 99,

            10 => 17,
            20 => 50,
        });
        let mut processor = Processor::new(memory);
        processor.execute().unwrap();

        let expected = Memory::new(memory! {
            0 => 2,
            1 => 10,
            2 => 20,
            3 => 100,
            4 => 99,

            10 => 17,
            20 => 50,
            100 => 17 * 50,
        });

        assert_eq!(processor.get_memory(), &expected);
    }

    #[test]
    fn invalid_opcode() {
        let memory = Memory::new(memory! {
            0 => 45,
        });

        let mut processor = Processor::new(memory);
        assert_eq!(processor.execute().unwrap_err(), Error::InvalidOpcode);
    }

    #[test]
    fn missing_terminator() {
        let memory = Memory::new(memory! {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 100,
        });
        assert_eq!(
            Processor::new(memory).execute(),
            Err(Error::FinishedWithoutTerminating)
        );
    }

    #[test]
    fn immediate_mode() {
        let memory = Memory::new(memory! {
            0 => 10_01,
            1 => 300,
            2 => 9,
            3 => 100,
            4 => 99,

            300 => 41,
        });
        let mut processor = Processor::new(memory);
        processor.execute().unwrap();

        let expected = Memory::new(memory! {
            0 => 10_01,
            1 => 300,
            2 => 9,
            3 => 100,
            4 => 99,

            100 => 50,
            300 => 41,
        });

        assert_eq!(processor.get_memory(), &expected);
    }

    #[test]
    fn input() {
        let memory = Memory::new(memory! {
            0 => 3,
            1 => 2,
            2 => 0,
            3 => 4,
            4 => 0,
        });

        let mut processor = Processor::new(memory);
        processor.push_input(Value(3));
        processor.push_input(Value(99));
        processor.execute().unwrap();
    }

    #[test]
    fn output() {
        let memory = Memory::new(memory! {
            0 => 1_04,
            1 => 50,
            2 => 99,
        });

        let mut processor = Processor::new(memory);
        let out = processor.execute_until_output().unwrap().unwrap();
        let term = processor.execute_until_output();

        assert_eq!(out, Value(50));
        assert_eq!(term, None);
    }

    #[test]
    fn jump_if_true() {
        let memory = Memory::new(memory! {
            0 => 11_05,
            1 => -90,
            2 => 100,
            100 => 99,
        });

        Processor::new(memory).execute().unwrap();
    }

    #[test]
    fn jump_if_false() {
        let memory = Memory::new(memory! {
            0 => 11_06,
            1 => 0,
            2 => 100,
            100 => 99,
        });

        Processor::new(memory).execute().unwrap();
    }

    #[test]
    fn less_than() {
        let memory = Memory::new(memory! {
            0 => 11_07,
            1 => -12,
            2 => 0,
            3 => 5,
            4 => 11_05,
            5 => 0,
            6 => 100,
            100 => 99,
        });

        Processor::new(memory).execute().unwrap();
    }

    #[test]
    fn equals() {
        let memory = Memory::new(memory! {
            0 => 11_08,
            1 => -12,
            2 => -12,
            3 => 5,
            4 => 11_05,
            5 => 0,
            6 => 100,
            100 => 99,
        });

        Processor::new(memory).execute().unwrap();
    }

    #[test]
    fn feedback_loop() {
        let memory = Memory::new(memory! {
            0 => 3, //IN -> input
            1 => 3,
            2 => 11_01, //ADD input 1 -> output
            3 => -99,
            4 => 1,
            5 => 7,
            6 => 1_04, //OUT -> output
            7 => -99,
            8 => 11_05, //JIT true 0
            9 => 1,
            10 => 0,
        });

        let mut processor = Processor::new(memory);
        let mut accumulator = Value(0);
        processor.push_input(accumulator);

        for _ in 0..10 {
            accumulator = processor
                .execute_until_output()
                .expect("Terminated")
                .expect("Errored");
            processor.push_input(accumulator);
        }
        assert_eq!(accumulator, Value(10))
    }

    #[test]
    fn relative_mode() {
        let memory = Memory::new(memory! {
            0 => 1_09, //ARB 10
            1 => 10,
            2 => 12_05, //JIT (rb + 5) 100
            3 => 5,
            4 => 100,
            100 => 99, //END

            15 => 1
        });

        Processor::new(memory).execute().unwrap();
    }
}
