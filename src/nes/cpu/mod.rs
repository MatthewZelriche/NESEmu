//! Emulation of the MOS Technologies 6502 CPU.
//!
//! While the console had several components operating in parallel, for emulation purposes the CPU can be
//! seen as driving the entire behavior of the system. It follows a simple Fetch-Decode-Execute loop and has
//! only a few registers and few dozen official instructions. Unofficial instructions are currently not
//! supported but are rarely utilized by official games.
//!
//! Due to the large size of the CPU's implemention, its impl block is split into multiple files for readability

use std::fmt::Display;

use bitfield::BitMut;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields,
    registers::InMemoryRegister,
};

use super::{bus::Bus, util::OptionalFile};

mod opcodes;

register_bitfields!(
    u8,
    pub Status [
        CARRY       OFFSET(0) NUMBITS(1) [],
        ZERO        OFFSET(1) NUMBITS(1) [],
        INT_DISABLE OFFSET(2) NUMBITS(1) [],
        DECIMAL     OFFSET(3) NUMBITS(1) [],    // Disabled, can be read and written to but does nothing
        BFLAG       OFFSET(4) NUMBITS(1) [],    // Not part of the physical hardware register, used only when
                                                // the register is pushed to the stack
        UNUSED      OFFSET(5) NUMBITS(1) [],
        OVERFLOW    OFFSET(6) NUMBITS(1) [],
        NEGATIVE    OFFSET(7) NUMBITS(1) [],
    ]
);

pub struct CPU {
    registers: CPURegisters,
    old_register_state: CPURegisters, // State for the CPU at the end of the PREVIOUS instruction
    total_cycles: usize,              // For debug printing only
    log_file: OptionalFile,
}

impl CPU {
    pub const PAGE_SZ_MASK: usize = 0xFF00;
    pub const STACK_PG_START: usize = 0x100;

    /// Constructs a new instance of the CPU
    ///
    /// Construction can fail if there is a failure to read the reset vector from the cartridge
    pub fn new(bus: &mut Bus) -> Result<Self, &'static str> {
        let mut this = Self {
            registers: CPURegisters::new(),
            old_register_state: CPURegisters::new(),
            total_cycles: 0,
            log_file: OptionalFile::new("nesemu.log"),
        };

        this.reset(bus)?;
        Ok(this)
    }

    /// Performs a reset of the CPU, for example in order to begin running a new cartridge
    pub fn reset(&mut self, bus: &mut Bus) -> Result<(), &'static str> {
        // Get start program counter from reset vector
        let mut buf = [0u8; 2];
        bus.cpu_read_exact(0xFFFC, &mut buf)?;
        self.registers.program_counter = u16::from_le_bytes(buf) as usize;

        self.total_cycles += 7;
        self.registers.stack_ptr -= 3;
        self.registers
            .status_register
            .modify(Status::INT_DISABLE::SET);
        Ok(())
    }

    /// Steps the CPU simulation by one instruction
    ///
    /// Note that this steps by an entire instruction, not by a single cycle. We play "catch-up" with the
    /// other components by stepping the CPU one instruction at a time, returning how many cycles that took,
    /// and then stepping the other components as necessary
    pub fn step(
        &mut self,
        bus: &mut Bus,
        pending_interrupt: &mut bool,
    ) -> Result<u8, &'static str> {
        // The nestest log requires the cpu register state PRIOR to executing
        // the instruction, so we copy the current state of the registers
        // for later, when we print to the log
        self.old_register_state = self.registers.clone();
        if *pending_interrupt {
            *pending_interrupt = false;
            return self.handle_irq(bus);
        }
        // Fetch the opcode
        let opcode = bus.cpu_read_byte(self.registers.program_counter)?;
        // We increment by one to skip over the opcode identifier byte, but processing the opcode
        // will handle adjusting the program counter to skip operand bytes
        self.registers.program_counter += 1;
        // TODO: Consider allowing debug logging of instructions via a keybind
        let cycle_count = self.execute_opcode(opcode, bus, false)?;
        self.total_cycles += cycle_count as usize;
        Ok(cycle_count)
    }

    /// Push bytes onto the stack, decrementing the stack pointer as necessary
    fn push_stack(&mut self, data: &[u8], bus: &mut Bus) -> Result<(), &'static str> {
        for byte in data {
            bus.cpu_write_byte(self.registers.stack_ptr + CPU::STACK_PG_START, *byte)?;
            self.registers.stack_ptr -= 1;
        }

        Ok(())
    }

    /// Pop bytes off of the stack, incrementing the stack pointer as necessary
    fn pop_stack(&mut self, data: &mut [u8], bus: &mut Bus) -> Result<(), &'static str> {
        for byte in &mut *data {
            self.registers.stack_ptr += 1;
            *byte = bus.cpu_read_byte(self.registers.stack_ptr + CPU::STACK_PG_START)?;
        }

        Ok(())
    }

    /// Sets the contents of the status register according to an 8 bit value
    ///
    /// The BREAK flag doesn't actually physically exist, so we always ignore it when setting the register's
    /// contents. Similarly, the unused bit must always be set to one
    fn set_status_register(&mut self, mut val: u8) {
        val.set_bit(4, self.registers.status_register.is_set(Status::BFLAG)); // Ignore value of BFLAG in stack
        val.set_bit(5, true); // Unused bit must always be set
        self.registers.status_register.set(val);
    }

    /// Instructs the CPU to handle an interrupt request
    pub fn handle_irq(&mut self, bus: &mut Bus) -> Result<u8, &'static str> {
        // TODO: This doesn't support IRQs which arent NMIs

        // Push the necessary bookkeeping information to return from interrupt vector onto the stack
        // big endian because we need to push to the stack in reverse order of how they should be
        self.push_stack(
            &u16::to_be_bytes((self.registers.program_counter) as u16),
            bus,
        )?;
        let sr = [self.registers.status_register.get()];
        self.registers
            .status_register
            .modify(Status::INT_DISABLE::SET);
        self.push_stack(&sr, bus)?;

        // Jump to the program's interrupt vector for the next instruction
        let mut interrupt_vector = [0u8; 2];
        bus.cpu_read_exact(0xFFFA, &mut interrupt_vector)?;
        self.registers.program_counter = u16::from_le_bytes(interrupt_vector) as usize;
        self.total_cycles += 7;
        Ok(8)
    }

    /// Checks if an adjustment to the program counter will cross a page boundary
    ///
    /// Some instructions take a variable number of cycles if the result of the instruction's execution
    /// causes the program counter to cross from one CPU memory page to another
    fn will_cross_boundary(old_pc: usize, new_pc: usize) -> bool {
        new_pc & CPU::PAGE_SZ_MASK != old_pc & CPU::PAGE_SZ_MASK
    }

    /// Convenience function to conditionally set a flag in the status register according to a predicate
    fn set_status_bit_if(&mut self, bit_pos: u8, predicate: bool) {
        let mut new = self.registers.status_register.get();
        new.set_bit(bit_pos.into(), predicate);
        self.registers.status_register.set(new);
    }
}

pub struct CPURegisters {
    pub accumulator: u8,
    pub x_reg: u8,
    pub y_reg: u8,
    pub stack_ptr: usize,
    pub program_counter: usize,
    pub status_register: InMemoryRegister<u8, Status::Register>,
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
    pub fn new() -> Self {
        Self {
            accumulator: Default::default(),
            x_reg: Default::default(),
            y_reg: Default::default(),
            // Technically starts at 0xFD, but calling new() on CPU triggers reset() which decrements by 3
            stack_ptr: 0xFF,
            program_counter: 0x0, // Will be set to the reset vector by reset(),
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
