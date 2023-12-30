use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::Write,
};

use tock_registers::{
    interfaces::{ReadWriteable, Readable},
    register_bitfields,
    registers::InMemoryRegister,
};

use super::bus::Bus;

register_bitfields!(
    u8,
    pub Status [
        CARRY       OFFSET(0) NUMBITS(1) [],
        ZERO        OFFSET(1) NUMBITS(1) [],
        INT_DISABLE OFFSET(2) NUMBITS(1) [],
        DECIMAL     OFFSET(3) NUMBITS(1) [],
        BFLAG       OFFSET(4) NUMBITS(1) [],
        UNUSED      OFFSET(5) NUMBITS(1) [],
        OVERFLOW    OFFSET(6) NUMBITS(1) [],
        NEGATIVE    OFFSET(6) NUMBITS(1) [],
    ]
);

pub struct CPURegisters {
    pub accumulator: u8,
    pub x_reg: u8,
    pub y_reg: u8,
    pub stack_ptr: usize,
    pub program_counter: usize,
    pub status_register: InMemoryRegister<u8, Status::Register>,
}

pub struct OptionalFile(Option<File>);

// This impl cannot fail
impl Write for OptionalFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(file) = self.0.as_mut() {
            let _ = write!(file, "{}", std::str::from_utf8(buf).unwrap()); // Don't care if fails...
            return Ok(buf.len());
        }

        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct CPU {
    pub registers: CPURegisters,
    cycles_remaining: u8,
    total_cycles: usize,
    pub log_file: OptionalFile,
}

impl CPU {
    pub fn new<T: Bus>(bus: &T) -> Result<Self, &'static str> {
        // Get start program counter
        let mut buf = [0u8; 2];
        bus.read_exact(0xFFFC, &mut buf)?;
        Ok(Self {
            registers: CPURegisters::new(u16::from_le_bytes(buf) as usize),
            cycles_remaining: 0,
            total_cycles: 7, // TODO: CPU init takes some prep work, not sure if I should step
            // through this or if its good enough to just set the
            // value instantly here
            log_file: OptionalFile(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open("nesemu.log")
                    .ok(),
            ),
        })
    }

    pub fn _reset(&mut self) {
        self.registers.stack_ptr -= 3;
        self.registers
            .status_register
            .modify(Status::INT_DISABLE::SET);
    }

    pub fn push_stack<T: Bus>(&mut self, data: &[u8], bus: &mut T) -> Result<(), &'static str> {
        for byte in data {
            bus.write_byte(self.registers.stack_ptr, *byte)?;
            self.registers.stack_ptr -= 1;
        }

        Ok(())
    }

    pub fn step<T: Bus>(&mut self, bus: &mut T) {
        if self.cycles_remaining != 0 {
            self.cycles_remaining -= 1;
        } else {
            // The nestest log requires the cpu register state PRIOR to executing
            // the instruction, so we copy the current state of the registers
            // for later, when we print to the log
            let old_state = self.registers.clone();

            // Fetch the opcode
            // Throw a BRK instruction is we can't read the opcode memory location
            // TODO: Better way of handling this?
            let opcode_addr = self.registers.program_counter;
            let opcode = bus.read_byte(opcode_addr).unwrap_or(0x0);
            self.registers.program_counter += 1;
            write!(self.log_file, "{:X}  ", opcode_addr).unwrap();

            match self.execute_opcode(opcode, bus) {
                Ok(cycle_count) => {
                    writeln!(
                        self.log_file,
                        "     {} CYC:{}",
                        old_state, self.total_cycles
                    )
                    .unwrap();
                    self.total_cycles += cycle_count as usize;
                    self.cycles_remaining = cycle_count;
                }
                Err(error) => {
                    log::error!(
                        "Instruction at address {:X} failed with msg: {}",
                        opcode_addr,
                        error
                    )
                }
            }
        }
    }
}

impl Clone for CPURegisters {
    fn clone(&self) -> Self {
        Self {
            accumulator: self.accumulator,
            x_reg: self.x_reg,
            y_reg: self.y_reg,
            stack_ptr: self.stack_ptr,
            program_counter: self.program_counter,
            status_register: InMemoryRegister::new(self.status_register.get()),
        }
    }
}

impl CPURegisters {
    pub fn new(reset_vector: usize) -> Self {
        Self {
            accumulator: Default::default(),
            x_reg: Default::default(),
            y_reg: Default::default(),
            stack_ptr: 0xFD,
            program_counter: reset_vector,
            status_register: InMemoryRegister::new(0x24), // Match nestest
        }
    }
}

impl Display for CPURegisters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.accumulator,
            self.x_reg,
            self.y_reg,
            self.status_register.get(),
            self.stack_ptr
        )
    }
}
