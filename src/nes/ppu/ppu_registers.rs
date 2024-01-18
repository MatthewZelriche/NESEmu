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
    pub PPUMASK [
        GRAYSCALE            OFFSET(0) NUMBITS(1) [],
        LEFT_8_MASK_BGRND    OFFSET(1) NUMBITS(1) [],
        LEFT_8_MASK_SPRTE    OFFSET(2) NUMBITS(1) [],
        SHOW_BACKGROUND      OFFSET(3) NUMBITS(1) [],
        SHOW_SPRITES         OFFSET(4) NUMBITS(1) [],
        EMPH_RED             OFFSET(5) NUMBITS(1) [],
        EMPH_GREEN           OFFSET(6) NUMBITS(1) [],
        EMPH_BLUE            OFFSET(7) NUMBITS(1) [],
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
    pub ppumask: InMemoryRegister<u8, PPUMASK::Register>,
    pub ppustatus: InMemoryRegister<u8, PPUSTATUS::Register>,
    pub ppuaddr: u16,
    pub ppudata: u8,
    pub write_latch: bool,
    pub fine_x: u8,
    pub fine_y: u8,
}

impl Default for PPURegisters {
    fn default() -> Self {
        Self {
            ppuctrl: InMemoryRegister::new(0),
            ppumask: InMemoryRegister::new(0),
            ppustatus: InMemoryRegister::new(0),
            ppuaddr: 0,
            ppudata: 0,
            fine_x: 0,
            fine_y: 0,
            write_latch: false,
        }
    }
}
