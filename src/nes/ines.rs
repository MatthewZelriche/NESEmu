use tock_registers::{register_bitfields, registers::InMemoryRegister};

register_bitfields! [
    u8,
    pub Flags1 [
        MIRRORING           OFFSET(0) NUMBITS(1) [
            HORZ = 0,
            VERT = 1,
        ],
        HAS_PRG_RAM         OFFSET(1) NUMBITS(1) [],
        HAS_TRAINER         OFFSET(2) NUMBITS(1) [],
        IGNORE_MIRRORING    OFFSET(3) NUMBITS(1) [],
        MAPPER_LOWER        OFFSET(4) NUMBITS(4) [],
    ]
];

register_bitfields! [
    u8,
    pub Flags2 [
        VS_UNISYSTEM        OFFSET(0) NUMBITS(1) [],
        PLAYCHOICE          OFFSET(1) NUMBITS(1) [],
        INES_VERSION        OFFSET(2) NUMBITS(2) [
            INES_20 = 2
        ],
        MAPPER_UPPER        OFFSET(4) NUMBITS(4) [],
    ]
];

pub struct INESHeader {
    pub prg_rom_size: u8,
    pub chr_rom_size: u8,
    pub flags1: InMemoryRegister<u8, Flags1::Register>,
    pub flags2: InMemoryRegister<u8, Flags2::Register>,
    pub prg_ram_size: u8,
    pub tv_system: u8,
}

impl Default for INESHeader {
    fn default() -> Self {
        Self {
            prg_rom_size: Default::default(),
            chr_rom_size: Default::default(),
            flags1: InMemoryRegister::new(0),
            flags2: InMemoryRegister::new(0),
            prg_ram_size: Default::default(),
            tv_system: Default::default(),
        }
    }
}
