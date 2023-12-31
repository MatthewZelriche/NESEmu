use eframe::epaint::Color32;

pub fn lookup_palette_color(idx: u8) -> Result<Color32, &'static str> {
    match idx {
        0x00 => Ok(Color32::from_rgb(98, 98, 98)),
        0x01 => Ok(Color32::from_rgb(0, 31, 178)),
        0x02 => Ok(Color32::from_rgb(36, 4, 200)),
        0x03 => Ok(Color32::from_rgb(82, 0, 178)),
        0x04 => Ok(Color32::from_rgb(115, 0, 118)),
        0x05 => Ok(Color32::from_rgb(128, 0, 36)),
        0x06 => Ok(Color32::from_rgb(115, 11, 0)),
        0x07 => Ok(Color32::from_rgb(82, 40, 0)),
        0x08 => Ok(Color32::from_rgb(36, 68, 0)),
        0x09 => Ok(Color32::from_rgb(0, 87, 0)),
        0x0A => Ok(Color32::from_rgb(0, 92, 0)),
        0x0B => Ok(Color32::from_rgb(0, 83, 36)),
        0x0C => Ok(Color32::from_rgb(0, 60, 118)),
        0x0D => Ok(Color32::from_rgb(0, 0, 0)),
        0x0E => Ok(Color32::from_rgb(0, 0, 0)),
        0x0F => Ok(Color32::from_rgb(0, 0, 0)),
        0x10 => Ok(Color32::from_rgb(171, 171, 171)),
        0x11 => Ok(Color32::from_rgb(13, 87, 255)),
        0x12 => Ok(Color32::from_rgb(75, 48, 255)),
        0x13 => Ok(Color32::from_rgb(138, 19, 255)),
        0x14 => Ok(Color32::from_rgb(118, 8, 214)),
        0x15 => Ok(Color32::from_rgb(210, 18, 105)),
        0x16 => Ok(Color32::from_rgb(199, 46, 0)),
        0x17 => Ok(Color32::from_rgb(157, 84, 0)),
        0x18 => Ok(Color32::from_rgb(96, 123, 0)),
        0x19 => Ok(Color32::from_rgb(32, 152, 0)),
        0x1A => Ok(Color32::from_rgb(0, 163, 0)),
        0x1B => Ok(Color32::from_rgb(0, 153, 66)),
        0x1C => Ok(Color32::from_rgb(0, 125, 180)),
        0x1D => Ok(Color32::from_rgb(0, 0, 0)),
        0x1E => Ok(Color32::from_rgb(0, 0, 0)),
        0x1F => Ok(Color32::from_rgb(0, 0, 0)),
        0x20 => Ok(Color32::from_rgb(255, 255, 255)),
        0x21 => Ok(Color32::from_rgb(83, 174, 255)),
        0x22 => Ok(Color32::from_rgb(144, 133, 255)),
        0x23 => Ok(Color32::from_rgb(211, 101, 255)),
        0x24 => Ok(Color32::from_rgb(255, 87, 255)),
        0x25 => Ok(Color32::from_rgb(255, 93, 207)),
        0x26 => Ok(Color32::from_rgb(255, 119, 87)),
        0x27 => Ok(Color32::from_rgb(250, 158, 0)),
        0x28 => Ok(Color32::from_rgb(189, 199, 0)),
        0x29 => Ok(Color32::from_rgb(122, 231, 0)),
        0x2A => Ok(Color32::from_rgb(67, 246, 17)),
        0x2B => Ok(Color32::from_rgb(38, 239, 126)),
        0x2C => Ok(Color32::from_rgb(44, 213, 246)),
        0x2D => Ok(Color32::from_rgb(78, 78, 78)),
        0x2E => Ok(Color32::from_rgb(0, 0, 0)),
        0x2F => Ok(Color32::from_rgb(0, 0, 0)),
        0x30 => Ok(Color32::from_rgb(255, 255, 255)),
        0x31 => Ok(Color32::from_rgb(182, 255, 255)),
        0x32 => Ok(Color32::from_rgb(206, 209, 255)),
        0x33 => Ok(Color32::from_rgb(233, 195, 255)),
        0x34 => Ok(Color32::from_rgb(255, 188, 255)),
        0x35 => Ok(Color32::from_rgb(255, 189, 244)),
        0x36 => Ok(Color32::from_rgb(255, 198, 195)),
        0x37 => Ok(Color32::from_rgb(255, 213, 154)),
        0x38 => Ok(Color32::from_rgb(233, 230, 129)),
        0x39 => Ok(Color32::from_rgb(206, 244, 129)),
        0x3A => Ok(Color32::from_rgb(182, 251, 154)),
        0x3B => Ok(Color32::from_rgb(169, 250, 195)),
        0x3C => Ok(Color32::from_rgb(169, 240, 244)),
        0x3D => Ok(Color32::from_rgb(184, 184, 184)),
        0x3E => Ok(Color32::from_rgb(0, 0, 0)),
        0x3F => Ok(Color32::from_rgb(0, 0, 0)),
        _ => Err("Invalid color palette idx"),
    }
}
