use super::Mode;
use super::{Error, Processor};
use crate::memory::{Address, Value};

use std::convert::TryInto;

#[cfg(debug_assertions)]
macro_rules! debug {
    ($x:expr) => {eprintln!("{}", $x)};

    ($x:expr, $($xs:expr),+) => {{
        eprint!("{} ", $x);
        debug!($($xs),+);
    }}
}
#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($($xs:expr),*) => {};
}

macro_rules! read_argument {
    ($processor:ident, $modes:expr, $index:expr) => {
        $processor.read_argument($modes[$index], Address($index))
    };
}

macro_rules! read_with_mode {
    ($processor:ident, $mode:expr, $index:expr) => {
        $processor.read_argument($mode, Address($index))
    };
}

type OpcodeResult = Result<Option<Value>, Error>;

impl Processor {
    pub(super) fn add(&mut self, modes: i32) -> OpcodeResult {
        let modes = split_modes::<2>(modes)?;
        let num1 = read_argument!(self, modes, 0)?;
        let num2 = read_argument!(self, modes, 1)?;
        let out_addr = read_with_mode!(self, Mode::Immediate, 2)?;
        let out_addr = out_addr.try_into()?;

        debug!("ADD", num1, num2, out_addr);

        self.memory.write(out_addr, Value(num1.0 + num2.0));
        self.pc = Address(self.pc.0 + 4);
        Ok(None)
    }

    pub(super) fn multiply(&mut self, modes: i32) -> OpcodeResult {
        let modes = split_modes::<2>(modes)?;
        let num1 = read_argument!(self, modes, 0)?;
        let num2 = read_argument!(self, modes, 1)?;
        let out_addr = read_with_mode!(self, Mode::Immediate, 2)?;
        let out_addr = out_addr.try_into()?;

        debug!("MUL", num1, num2, out_addr);

        self.memory.write(out_addr, Value(num1.0 * num2.0));
        self.pc = Address(self.pc.0 + 4);
        Ok(None)
    }

    pub(super) fn input(&mut self) -> OpcodeResult {
        let out_addr = read_with_mode!(self, Mode::Immediate, 0)?;
        let out_addr = TryInto::<Address>::try_into(out_addr)?;

        debug!("IN", out_addr);

        let input = self.input_buffer.pop_front().ok_or(Error::InputReadError)?;

        self.memory.write(out_addr, input);
        self.pc = Address(self.pc.0 + 2);
        Ok(None)
    }

    pub(super) fn output(&mut self, modes: i32) -> OpcodeResult {
        let modes = split_modes::<1>(modes)?;
        let val = read_argument!(self, modes, 0)?;

        debug!("OUT", val);

        self.pc = Address(self.pc.0 + 2);
        Ok(Some(val))
    }

    pub(super) fn jump_if_true(&mut self, modes: i32) -> OpcodeResult {
        let modes = split_modes::<2>(modes)?;
        let condition = read_argument!(self, modes, 0)?;
        let jump_to = read_argument!(self, modes, 1)?;
        let jump_to = jump_to.try_into()?;

        debug!("JIT", condition, jump_to);

        self.pc = if condition.0 != 0 {
            jump_to
        } else {
            Address(self.pc.0 + 3)
        };
        Ok(None)
    }

    pub(super) fn jump_if_false(&mut self, modes: i32) -> OpcodeResult {
        let modes = split_modes::<2>(modes)?;
        let condition = read_argument!(self, modes, 0)?;
        let jump_to = read_argument!(self, modes, 1)?;
        let jump_to = jump_to.try_into()?;

        debug!("JIF", condition, jump_to);

        self.pc = if condition.0 == 0 {
            jump_to
        } else {
            Address(self.pc.0 + 3)
        };
        Ok(None)
    }

    pub(super) fn less_than(&mut self, modes: i32) -> OpcodeResult {
        let modes = split_modes::<3>(modes)?;
        let cmp1 = read_argument!(self, modes, 0)?;
        let cmp2 = read_argument!(self, modes, 1)?;
        let out_addr = read_with_mode!(self, Mode::Immediate, 2)?;

        debug!("LT", cmp1, cmp2, out_addr);

        self.memory.write(
            out_addr.try_into()?,
            Value(if cmp1.0 < cmp2.0 { 1 } else { 0 }),
        );

        self.pc = Address(self.pc.0 + 4);
        Ok(None)
    }

    pub(super) fn equals(&mut self, modes: i32) -> OpcodeResult {
        let modes = split_modes::<3>(modes)?;
        let cmp1 = read_argument!(self, modes, 0)?;
        let cmp2 = read_argument!(self, modes, 1)?;
        let out_addr = read_with_mode!(self, Mode::Immediate, 2)?;

        debug!("EQ", cmp1, cmp2, out_addr);

        self.memory.write(
            out_addr.try_into()?,
            Value(if cmp1.0 == cmp2.0 { 1 } else { 0 }),
        );

        self.pc = Address(self.pc.0 + 4);
        Ok(None)
    }

    fn read_argument(&self, mode: Mode, index: Address) -> Result<Value, Error> {
        let addr = Address(self.pc.0 + 1 + index.0);
        let out = Ok(match mode {
            Mode::Positional => {
                let addr2 = self.memory.read(addr)?;
                self.memory.read(addr2.try_into()?)?
            }
            Mode::Immediate => self.memory.read(addr)?,
        });
        out
    }
}

fn split_modes<const N: usize>(num: i32) -> Result<[Mode; N], Error> {
    let mut out: [Option<Mode>; N] = [None; N];

    let mut modulor = 10;
    let mut divisor = 1;
    for index in 0..N {
        let mode = (num % modulor) / divisor;
        modulor *= 10;
        divisor *= 10;

        out[index] = Some(mode.try_into().or(Err(Error::IllegalMode))?);
    }
    let out = out.map(|opt_mode| opt_mode.unwrap());
    Ok(out)
}
