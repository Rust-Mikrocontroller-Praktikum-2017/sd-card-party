pub fn four_bytes_at_offset(block: &[u8], offset: usize) -> u32 {
    let first: u32 = block[offset] as u32;
    let second: u32 = block[offset + 1] as u32;
    let third: u32 = block[offset + 2] as u32;
    let fourth: u32 = block[offset + 3] as u32;
    (first | second << 8 | third << 16 | fourth << 24)
}

pub fn two_bytes_at_offset(block: &[u8], offset: usize) -> u16 {
    let first: u16 = block[offset] as u16;
    let second: u16 = block[offset + 1] as u16;
    (first | second << 8)
}
