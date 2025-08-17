use alloc::string::ToString;

use aidoku::{
	Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, Page,
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::format,
	std::{ObjectRef, String, Vec},
};
use time::format_description::well_known::Rfc3339;

use crate::BASE_URL;

pub fn parse_search_page(data: ObjectRef, page: i32) -> Result<MangaPageResult> {
	let mangas = if let Ok(arr) = data.get("data").as_array() {
		arr
	} else {
		data.get("data").as_object()?.get("mangas").as_array()?
	};

	Ok(MangaPageResult {
		manga: mangas
			.map(|manga| {
				let item = manga.as_object().unwrap();
				Manga {
					id: item.get("id").as_int().unwrap().to_string(),
					cover: item.get("cover_mobile_url").as_string().unwrap().read(),
					title: item.get("name").as_string().unwrap().read(),
					author: item
						.get("author_name")
						.as_string()
						.unwrap_or_default()
						.read(),
					description: item
						.get("newest_chapter_number")
						.as_string()
						.unwrap_or_default()
						.read(),
					..Default::default()
				}
			})
			.collect(),
		has_more: page
			< data
				.get("_metadata")
				.as_object()
				.unwrap()
				.get("total_pages")
				.as_int()
				.unwrap() as i32,
	})
}

pub fn parse_manga_details(data: ObjectRef) -> Result<Manga> {
	let meta = data.get("data").as_object().unwrap();

	let id = meta.get("id").as_int().unwrap().to_string();
	let cover = meta.get("cover_mobile_url").as_string().unwrap().read();
	let title = meta.get("name").as_string().unwrap().read();
	let author = meta
		.get("author")
		.as_object()
		.unwrap()
		.get("name")
		.as_string()
		.unwrap()
		.read();
	let description = meta.get("full_description").as_string().unwrap().read();
	let url = format!("{BASE_URL}/mangas/${id}");
	let categories = meta
		.get("tags")
		.as_array()
		.unwrap()
		.map(|tag_ref| {
			let tag = tag_ref.as_object().unwrap();

			tag.get("name").as_string().unwrap().read()
		})
		.collect();
	let status = MangaStatus::Unknown;
	let nsfw = if meta.get("is_nsfw").as_bool().unwrap() {
		MangaContentRating::Nsfw
	} else {
		MangaContentRating::Safe
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		description,
		url,
		categories,
		status,
		nsfw,
		..Default::default()
	})
}

pub fn parse_chapter_list(data: ObjectRef, manga_id: String) -> Result<Vec<Chapter>> {
	Ok(data
		.get("data")
		.as_array()
		.unwrap()
		.map(|chapter_ref| {
			let chapter = chapter_ref.as_object().unwrap();

			let id = chapter.get("id").as_int().unwrap().to_string();

			Chapter {
				id: id.clone(),
				title: chapter.get("name").as_string().unwrap().read(),
				volume: -1.0,
				chapter: chapter
					.get("number")
					.as_string()
					.and_then(|s| {
						s.read()
							.to_string()
							.parse::<f32>()
							.map_err(|_| AidokuError {
								reason: AidokuErrorKind::JsonParseError,
							})
					})
					.unwrap_or(-1.0),
				date_updated: chapter
					.get("updated_at")
					.as_string()
					.and_then(|s| {
						time::OffsetDateTime::parse(&s.read(), &Rfc3339).map_err(|_| AidokuError {
							reason: AidokuErrorKind::JsonParseError,
						})
					})
					.map(|dt| dt.unix_timestamp() as f64)
					.unwrap_or(-1.0),
				url: format!("{BASE_URL}/mangas/{manga_id}/chapters/{id}"),
				lang: "en".to_string(),
				..Default::default()
			}
		})
		.collect())
}

pub fn parse_page_list(data: ObjectRef) -> Result<Vec<Page>> {
	let chapters = data
		.get("data")
		.as_object()
		.unwrap()
		.get("pages")
		.as_array()
		.unwrap();

	if chapters.clone().count() == 0 {
		return Err(AidokuError {
			reason: AidokuErrorKind::DefaultNotFound,
		});
	};

	let pages = chapters
		.map(|chapter_ref| {
			let chapter = chapter_ref.as_object().unwrap();

			let index = chapter.get("order").as_int().unwrap() as i32;
			let url = chapter.get("image_url").as_string().unwrap().read();
			let drm_data = chapter.get("drm_data").as_string().unwrap().read();

			let blocks = decode_drm(drm_data.replace('\n', "").trim()).unwrap();
			let base64 = {
				let mut s = String::from("[");
				for (i, rb) in blocks.iter().enumerate() {
					if i > 0 {
						s.push(',');
					}
					s.push_str(&format!(
						r#"{{"sx":0,"sy":-1,"dx":0,"dy":{},"width":0,"height":{}}}"#,
						rb.dy, rb.height
					));
				}
				s.push(']');
				s
			};

			Page {
				index,
				url,
				base64,
				..Default::default()
			}
		})
		.collect();

	Ok(pages)
}

#[derive(Debug, Clone)]
pub struct RowBlock {
	pub dy: i32,
	pub height: i32,
}

fn decode_xor_cipher(data: &[u8]) -> Vec<u8> {
	let key_bytes: [u8; 16] = [
		51, 49, 52, 49, 53, 57, 50, 54, 53, 51, 53, 56, 57, 55, 57, 51,
	];
	let mut output: Vec<u8> = Vec::with_capacity(data.len());

	for (i, &b) in data.iter().enumerate() {
		let kb = key_bytes[i % key_bytes.len()];
		let x = b ^ kb;
		output.push(x);
	}

	output
}

fn decode_drm(drm_data: &str) -> Result<Vec<RowBlock>> {
	let decoded = decode_base64(&drm_data.replace('\n', ""))
		.map_err(|_| "Base64 decode failed")
		.unwrap();
	let xored = decode_xor_cipher(&decoded);

	let decoded_str = str::from_utf8(&xored).map_err(|_| "Invalid UTF-8").unwrap();

	if !decoded_str.starts_with("#v4|") {
		// return Err("Invalid DRM data (does not start with expected magic bytes)");

		return Err(AidokuError {
			reason: AidokuErrorKind::JsonParseError,
		});
	}

	let parts: Vec<&str> = decoded_str.split('|').skip(1).collect();
	let mut blocks = Vec::new();

	for part in parts {
		let values: Vec<&str> = part.split('-').collect();
		if values.len() == 2 {
			let dy: i32 = values[0].parse().unwrap();
			let height: i32 = values[1].parse().unwrap();
			blocks.push(RowBlock { dy, height });
		}
	}

	Ok(blocks)
}

fn decode_base64(input: &str) -> Result<Vec<u8>> {
	let mut out = Vec::with_capacity(input.len() * 3 / 4 + 1);
	let mut buffer: u32 = 0;
	let mut bits_collected = 0;

	for &b in input.as_bytes() {
		let val = match b {
			b'A'..=b'Z' => b - b'A',
			b'a'..=b'z' => b - b'a' + 26,
			b'0'..=b'9' => b - b'0' + 52,
			b'+' => 62,
			b'/' => 63,
			b'=' => 0,
			b'\n' | b'\r' | b'\t' | b' ' => continue,
			_ => {
				return Err(AidokuError {
					reason: AidokuErrorKind::DefaultNotFound,
				});
			}
		} as u32;

		buffer = (buffer << 6) | val;
		bits_collected += 6;

		if bits_collected >= 8 {
			bits_collected -= 8;
			let byte = (buffer >> bits_collected) as u8;
			out.push(byte);
			buffer &= (1 << bits_collected) - 1;
		}
	}

	Ok(out)
}
