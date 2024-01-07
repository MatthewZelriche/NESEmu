use bitfield::Bit;

pub struct InputEvent {
    pub input_state: u8,
}

impl InputEvent {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const SELECT: u8 = 2;
    pub const START: u8 = 3;
    pub const UP: u8 = 4;
    pub const DOWN: u8 = 5;
    pub const LEFT: u8 = 6;
    pub const RIGHT: u8 = 7;
    pub const END: u8 = 8;
}

pub struct Controller {
    serial: bool,
    input_state: u8,
    return_bit: u8,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            serial: true,
            input_state: 0,
            return_bit: InputEvent::A,
        }
    }

    pub fn set_state_from_window(&mut self, event: InputEvent) {
        self.input_state = event.input_state;
    }

    pub fn write_to_controller(&mut self, serial: bool) {
        self.serial = serial;
        self.return_bit = InputEvent::A;
    }

    pub fn read_from_controller(&mut self) -> u8 {
        let res = u8::from(self.input_state.bit(self.return_bit as usize));
        if !self.serial {
            if self.return_bit == InputEvent::END {
                return 1;
            }
            self.return_bit += 1;
        }

        res
    }
}
