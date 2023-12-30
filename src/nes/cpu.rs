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

use super::{bus::Bus, instruction::DecodedInstruction};

register_bitfields!(
    u8,
    Status [
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

pub struct CPU {
    pub registers: CPURegisters,
    current_instruction: Option<DecodedInstruction>,
    total_cycles: usize,
    log_file: Option<File>,
}

impl CPU {
    pub fn new<T: Bus>(bus: &T) -> Result<Self, &'static str> {
        // Get start program counter
        let mut buf = [0u8; 2];
        bus.read_exact(0xFFFC, &mut buf)?;
        Ok(Self {
            registers: CPURegisters::new(u16::from_le_bytes(buf) as usize),
            current_instruction: None,
            total_cycles: 7, // TODO: CPU init takes some prep work, not sure if I should step
            // through this or if its good enough to just set the
            // value instantly here
            log_file: OpenOptions::new()
                .append(true)
                .create(true)
                .open("nesemu.log")
                .ok(),
        })
    }

    pub fn _reset(&mut self) {
        self.registers.stack_ptr -= 3;
        self.registers
            .status_register
            .modify(Status::INT_DISABLE::SET);
    }

    pub fn step<T: Bus>(&mut self, bus: &mut T) {
        match self.current_instruction.as_mut() {
            Some(instruction) => {
                instruction.cycles_remaining -= 1;
                if instruction.cycles_remaining == 0 {
                    self.current_instruction = None;
                }
            }
            None => {
                // The nestest log requires the cpu register state PRIOR to executing
                // the instruction, so we copy the current state of the registers
                // for later, when we print to the log
                let old_state = self.registers.clone();
                let old_total_cycles = self.total_cycles;

                // Fetch the opcode
                // Throw a BRK instruction is we can't read the opcode memory location
                // TODO: Better way of handling this?
                let opcode_addr = self.registers.program_counter;
                let opcode = bus.read_byte(opcode_addr).unwrap_or(0x0);
                self.registers.program_counter += 1;

                match self.execute_opcode(opcode, bus) {
                    Ok(instruction) => {
                        self.log_instruction(
                            opcode_addr,
                            &instruction,
                            &old_state,
                            old_total_cycles,
                        );
                        self.current_instruction = Some(instruction);
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

    pub fn log_instruction(
        &mut self,
        opcode_addr: usize,
        instruction: &DecodedInstruction,
        old_state: &CPURegisters,
        old_total_cycles: usize,
    ) {
        let fmt = format!(
            "{:X}  {}     {}   CYC:{}",
            opcode_addr, instruction, old_state, old_total_cycles
        );
        log::info!("{}", fmt);

        if let Some(log) = &mut self.log_file {
            let _ = log.write(fmt.as_bytes()); // Don't care much if it fails...
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
            "A: {:02X} X: {:02X} Y: {:02X} P: {:02X} SP: {:02X}",
            self.accumulator,
            self.x_reg,
            self.y_reg,
            self.status_register.get(),
            self.stack_ptr
        )
    }
}
