use tock_registers::{register_bitfields, registers::InMemoryRegister};

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

pub struct CPU {
    accumulator: u8,
    x_reg: u8,
    y_reg: u8,
    stack_ptr: usize,
    program_counter: usize,
    status_register: InMemoryRegister<u8, Status::Register>,
}

impl CPU {}

impl Default for CPU {
    fn default() -> Self {
        Self {
            accumulator: Default::default(),
            x_reg: Default::default(),
            y_reg: Default::default(),
            stack_ptr: 0xFD,
            program_counter: Default::default(),
            status_register: InMemoryRegister::new(0x24), // Match nestest
        }
    }
}
