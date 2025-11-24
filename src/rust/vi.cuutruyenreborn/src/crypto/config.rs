use spin::Once;

use super::env;

pub const GCM_NONCE_LEN: usize = 12;
pub const GCM_TAG_LEN: usize = 16; // implicit in AES-GCM ciphertext

// rotate right byte (reverse)
fn ror_byte(byte: u8, shift: u8) -> u8 {
	byte.rotate_right(shift as u32)
}

// ----------------- MASTER KEY -----------------
static MASTER_BYTES: Once<[u8; 32]> = Once::new();
pub fn get_master_key_bytes() -> &'static [u8; 32] {
	MASTER_BYTES.call_once(|| {
		assert_eq!(
			env::MASTER_KEY_HASH_SHIFTED.len(),
			32,
			"MASTER_KEY_HASH must be 32 bytes"
		);
		let mut arr = [0u8; 32];
		for (i, b) in env::MASTER_KEY_HASH_SHIFTED.iter().enumerate() {
			arr[i] = ror_byte(*b, env::SHIFT_KEY);
		}
		arr
	})
}

// ----------------- SHIFT VALUE -----------------
pub fn get_shift_value() -> u8 {
	env::SHIFT_KEY
}
