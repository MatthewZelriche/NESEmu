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
        DECIMAL     OFFSET(3) NUMBITS(1) [],
        BFLAG       OFFSET(4) NUMBITS(1) [],
        UNUSED      OFFSET(5) NUMBITS(1) [],
        OVERFLOW    OFFSET(6) NUMBITS(1) [],
        NEGATIVE    OFFSET(7) NUMBITS(1) [],
    ]
);

pub struct CPU {
    registers: CPURegisters,
    old_register_state: CPURegisters, // State for the CPU at the end of the PREVIOUS instruction
    current_instruction_addr: usize,  // Stores the instruction of the opcode currently executing
    cycles_remaining: u8,
    total_cycles: usize,
    log_file: OptionalFile,
}

impl CPU {
    pub const PAGE_SZ_MASK: usize = 0xFF00;
    pub const STACK_PG_START: usize = 0x100;

    pub fn new(bus: &mut Bus) -> Result<Self, &'static str> {
        // Get start program counter
        let mut buf = [0u8; 2];
        bus.cpu_read_exact(0xFFFC, &mut buf)?;
        Ok(Self {
            registers: CPURegisters::new(u16::from_le_bytes(buf) as usize),
            old_register_state: CPURegisters::new(u16::from_le_bytes(buf) as usize),
            current_instruction_addr: 0x0,
            cycles_remaining: 0,
            total_cycles: 7, // TODO: CPU init takes some prep work, not sure if I should step
            // through this or if its good enough to just set the
            // value instantly here
            log_file: OptionalFile::new("nesemu.log"),
        })
    }

    pub fn _reset(&mut self) {
        self.registers.stack_ptr -= 3;
        self.registers
            .status_register
            .modify(Status::INT_DISABLE::SET);
    }

    fn push_stack(&mut self, data: &[u8], bus: &mut Bus) -> Result<(), &'static str> {
        for byte in data {
            bus.cpu_write_byte(self.registers.stack_ptr + CPU::STACK_PG_START, *byte)?;
            self.registers.stack_ptr -= 1;
        }

        Ok(())
    }

    fn pop_stack(&mut self, data: &mut [u8], bus: &mut Bus) -> Result<(), &'static str> {
        for byte in &mut *data {
            self.registers.stack_ptr += 1;
            *byte = bus.cpu_read_byte(self.registers.stack_ptr + CPU::STACK_PG_START)?;
        }

        Ok(())
    }

    fn set_status_register(&mut self, mut val: u8) {
        val.set_bit(4, self.registers.status_register.is_set(Status::BFLAG)); // Ignore value of BFLAG in stack
        val.set_bit(5, true); // Unused bit must always be set
        self.registers.status_register.set(val);
    }

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
        self.current_instruction_addr = self.registers.program_counter;
        let opcode = bus.cpu_read_byte(self.current_instruction_addr)?;
        self.registers.program_counter += 1;
        let cycle_count = self.execute_opcode(opcode, bus, false)?;
        self.total_cycles += cycle_count as usize;
        self.cycles_remaining = cycle_count - 1;
        Ok(cycle_count)
    }

    pub fn handle_irq(&mut self, bus: &mut Bus) -> Result<u8, &'static str> {
        // TODO: This doesn't support IRQs which arent NMIs

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

        let mut interrupt_vector = [0u8; 2];
        bus.cpu_read_exact(0xFFFA, &mut interrupt_vector)?;
        self.registers.program_counter = u16::from_le_bytes(interrupt_vector) as usize;
        self.total_cycles += 7;
        Ok(8)
    }

    fn will_cross_boundary(old_pc: usize, new_pc: usize) -> bool {
        new_pc & CPU::PAGE_SZ_MASK != old_pc & CPU::PAGE_SZ_MASK
    }

    fn set_flag_bit_if(&mut self, bit_pos: u8, predicate: bool) {
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
