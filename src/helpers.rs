pub fn add_le_bytes(a: [u8; 8], b: [u8; 8]) -> [u8; 8] {
    let val_a = u64::from_le_bytes(a);
    let val_b = u64::from_le_bytes(b);
    val_a.checked_add(val_b).unwrap().to_le_bytes()
}
