use super::config::{GCM_NONCE_LEN, GCM_TAG_LEN, get_master_key_bytes};
use crate::alloc::string::ToString;
use aes_gcm::{
	Aes256Gcm, Nonce,
	aead::{Aead, KeyInit},
};
use aidoku::std::{String, Vec};
use base64::{Engine as _, engine::general_purpose};

pub fn decode_b64(s: &str) -> Result<Vec<u8>, String> {
	let mut padded = s.to_string();
	let rem = s.len() % 4;
	if rem != 0 {
		let pad_len = 4 - rem;
		padded.push_str(&"=".repeat(pad_len));
	}

	general_purpose::STANDARD
		.decode(&padded)
		.map_err(|_| "Invalid base64".into())
}

pub fn unwrap_key_with_master_internal(decoded: Vec<u8>) -> Result<Vec<u8>, String> {
	let master = get_master_key_bytes();
	if decoded.len() < GCM_NONCE_LEN + GCM_TAG_LEN {
		return Err("Wrapped key data too short".into());
	}
	let (nonce_bytes, rest) = decoded.split_at(GCM_NONCE_LEN);

	let cipher = Aes256Gcm::new_from_slice(master).map_err(|_| "Master cipher init failed")?;
	let mut nonce = Nonce::default();
	nonce.clone_from_slice(nonce_bytes);

	cipher
		.decrypt(&nonce, rest)
		.map_err(|_| "Master unwrap failed".into())
}

pub fn unshift_inplace(buf: &mut [u8], shift: u8) {
	if shift == 0 {
		return;
	}
	for b in buf.iter_mut() {
		*b = b.wrapping_sub(shift);
	}
}
