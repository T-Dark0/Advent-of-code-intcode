use super::{Error, Mode, Modes, Processor};
use crate::memory::{Address, Value};

use std::convert::{TryFrom, TryInto};

type OpcodeResult = Result<Option<Value>, Error>;

impl<I> Processor<I>
where
    I: Iterator<Item = Value>,
{
    pub(super) fn add(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<3>(modes)?;
        let args = self.read_arguments::<2>(&modes)?;

        let res = args[0] + args[1];

        self.write_result(&modes, 2, res)?;
        self.pc = Address(self.pc.0 + 4);
        Ok(None)
    }

    pub(super) fn multiply(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<3>(modes)?;
        let args = self.read_arguments::<2>(&modes)?;

        let res = args[0] * args[1];

        self.write_result(&modes, 2, res)?;
        self.pc = Address(self.pc.0 + 4);
        Ok(None)
    }

    pub(super) fn input(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<1>(modes)?;

        let res = self.input.next().ok_or(Error::InputReadError)?;

        self.write_result(&modes, 0, res)?;
        self.pc = Address(self.pc.0 + 2);
        Ok(None)
    }

    pub(super) fn output(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<1>(modes)?;
        let args = self.read_arguments::<1>(&modes)?;

        self.pc = Address(self.pc.0 + 2);
        Ok(Some(args[0]))
    }

    pub(super) fn jump_if_true(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<2>(modes)?;
        let args = self.read_arguments::<2>(&modes)?;

        self.pc = if args[0].0 != 0 {
            args[1].try_into()?
        } else {
            Address(self.pc.0 + 3)
        };
        Ok(None)
    }

    pub(super) fn jump_if_false(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<2>(modes)?;
        let args = self.read_arguments::<2>(&modes)?;

        self.pc = if args[0].0 == 0 {
            args[1].try_into()?
        } else {
            Address(self.pc.0 + 3)
        };
        Ok(None)
    }

    pub(super) fn less_than(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<3>(modes)?;
        let args = self.read_arguments::<2>(&modes)?;

        let res = Value(if args[0].0 < args[1].0 { 1 } else { 0 });

        self.write_result(&modes, 2, res)?;
        self.pc = Address(self.pc.0 + 4);
        Ok(None)
    }

    pub(super) fn equals(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<3>(modes)?;
        let args = self.read_arguments::<2>(&modes)?;

        let res = Value(if args[0].0 == args[1].0 { 1 } else { 0 });

        self.write_result(&modes, 2, res)?;
        self.pc = Address(self.pc.0 + 4);
        Ok(None)
    }

    pub(super) fn adjust_relative_base(&mut self, modes: Modes) -> OpcodeResult {
        let modes = split_modes::<1>(modes)?;
        let args = self.read_arguments::<1>(&modes)?;

        self.relative_base = Value(self.relative_base.0 + args[0].0);
        self.pc = Address(self.pc.0 + 2);
        Ok(None)
    }

    fn read_arguments<const N: usize>(&self, modes: &[Mode]) -> Result<[Value; N], Error> {
        let mut out = [None; N];
        for index in 0..N {
            let addr = Address(index.try_into().unwrap());
            out[index] = Some(self.read_argument(modes[index], addr)?);
        }
        Ok(out.map(Option::unwrap))
    }

    fn write_result(
        &mut self,
        modes: &[Mode],
        arg_index: usize,
        value: Value,
    ) -> Result<(), Error> {
        let mode = modes[arg_index];
        let out_arg_addr = Address(self.pc.0 + 1 + u32::try_from(arg_index).unwrap());
        let immediate_out = self.memory.read(out_arg_addr);
        match mode {
            Mode::Positional => self.memory.write(immediate_out.try_into()?, value),
            Mode::Immediate => self.memory.write(out_arg_addr, value),
            Mode::Relative => self
                .memory
                .write((self.relative_base + immediate_out).try_into()?, value),
        }
        Ok(())
    }

    fn read_argument(&self, mode: Mode, index: Address) -> Result<Value, Error> {
        let addr = Address(self.pc.0 + 1 + index.0);
        let addr2 = self.memory.read(addr);
        let out = match mode {
            Mode::Positional => self.memory.read(addr2.try_into()?),
            Mode::Immediate => addr2,
            Mode::Relative => self.memory.read((self.relative_base + addr2).try_into()?),
        };
        Ok(out)
    }
}

fn split_modes<const N: usize>(num: Modes) -> Result<[Mode; N], Error> {
    let mut out: [Option<Mode>; N] = [None; N];

    let mut modulor = 10;
    let mut divisor = 1;
    for index in 0..N {
        let mode = (num % modulor) / divisor;
        modulor *= 10;
        divisor *= 10;

        out[index] = Some(mode.try_into().or(Err(Error::IllegalMode))?);
    }
    let out = out.map(Option::unwrap);
    Ok(out)
}
