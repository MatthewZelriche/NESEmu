use tock_registers::{register_bitfields, registers::InMemoryRegister};

register_bitfields!(
    u8,
    pub PPUCTRL [
        NTABLE_ADDR         OFFSET(0) NUMBITS(2) [
            Addr2000 = 0,
            Addr2400 = 1,
            Addr2800 = 2,
            Addr2C00 = 3,
        ],
        VRAM_INC            OFFSET(2) NUMBITS(1) [],
        SPTNTABLE_ADDR      OFFSET(3) NUMBITS(1) [],
        BPTNTABLE_ADDR      OFFSET(4) NUMBITS(1) [],
        SPRITE_SIZE         OFFSET(5) NUMBITS(1) [],
        MASTER_SLAVE_SELECT OFFSET(6) NUMBITS(1) [],
        NMI_ENABLE          OFFSET(7) NUMBITS(1) [],
    ]
);

register_bitfields!(
    u8,
    pub PPUSTATUS [
        UNUSED              OFFSET(0) NUMBITS(5) [],
        SPRITE_OVERFLOW     OFFSET(5) NUMBITS(1) [],
        SPRITE0_HIT         OFFSET(6) NUMBITS(1) [],
        VBLANK              OFFSET(7) NUMBITS(1) [],
    ]
);

pub struct PPURegisters {
    pub ppuctrl: InMemoryRegister<u8, PPUCTRL::Register>,
    pub ppustatus: InMemoryRegister<u8, PPUSTATUS::Register>,
}

impl Default for PPURegisters {
    fn default() -> Self {
        Self {
            ppuctrl: InMemoryRegister::new(0),
            ppustatus: InMemoryRegister::new(0),
        }
    }
}
