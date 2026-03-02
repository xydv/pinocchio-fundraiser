pub fn add_le_bytes(a: [u8; 8], b: [u8; 8]) -> [u8; 8] {
    let val_a = u64::from_le_bytes(a);
    let val_b = u64::from_le_bytes(b);
    val_a.checked_add(val_b).unwrap().to_le_bytes()
}

pub fn sub_le_bytes(a: [u8; 8], b: [u8; 8]) -> [u8; 8] {
    let val_a = u64::from_le_bytes(a);
    let val_b = u64::from_le_bytes(b);
    if val_a < val_b {
        return val_b.checked_sub(val_a).unwrap().to_le_bytes();
    };
    val_a.checked_sub(val_b).unwrap().to_le_bytes()
}
