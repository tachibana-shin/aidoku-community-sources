use super::config::get_shift_value;
use super::crypto::{decode_b64, unshift_inplace, unwrap_key_with_master_internal};
use aes_gcm::aead::Aead as _;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aidoku::std::String;

pub fn decrypt_data_with_wrapped_key(
	wrapped_key_b64: String,
	data_b64: String,
) -> Result<String, String> {
	// Unwrap key
	let mut key_buff = decode_b64(&wrapped_key_b64)?;
	unshift_inplace(&mut key_buff, get_shift_value());

	let key = unwrap_key_with_master_internal(key_buff)?;
	if key.len() != 32 {
		return Err("Unwrapped key length is not 32 bytes".into());
	}

	// Decode data
	let mut data = decode_b64(&data_b64)?;
	unshift_inplace(&mut data, get_shift_value());
	if data.len() < super::config::GCM_NONCE_LEN {
		return Err("Data too short".into());
	}
	let (nonce_bytes, ciphertext_and_tag) = data.split_at(super::config::GCM_NONCE_LEN);

	let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "Cipher init failed")?;
	let mut nonce = Nonce::default();
	nonce.clone_from_slice(nonce_bytes);

	let plaintext = cipher
		.decrypt(&nonce, ciphertext_and_tag)
		.map_err(|_| "Decryption failed")?;
	String::from_utf8(plaintext).map_err(|_| "Invalid UTF-8 plaintext".into())
}
