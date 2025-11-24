use alloc::borrow::ToOwned;
use alloc::string::ToString;

use crate::crypto::main::decrypt_data_with_wrapped_key;
use aidoku::{
	Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, Page,
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::format,
	std::{ArrayRef, ObjectRef, String, Vec},
};
use time::format_description::well_known::Rfc3339;

use crate::{
	BASE_URL,
	utils::{get_list_name, parse_title, resolve_image_url},
};

pub fn parse_search_page(data: ObjectRef, page: i32) -> Result<MangaPageResult> {
	let mangas = if let Ok(arr) = data.get("items").as_array() {
		arr
	} else {
		data.get("data").as_object()?.get("items").as_array()?
	};

	Ok(MangaPageResult {
		manga: mangas
			.map(|manga| {
				let item = manga.as_object().unwrap();
				Manga {
					id: item.get("id").as_int().unwrap().to_string(),
					cover: resolve_image_url(
						&item.get("cover_mobile_url").as_string().unwrap().read(),
					),
					title: item.get("title_name").as_string().unwrap().read(),
					author: get_list_name(&item.get("authors").as_array().unwrap()).join(", "),
					description: "Ch.".to_owned()
						+ &item
							.get("last_chapter_number")
							.as_string()
							.unwrap_or_default()
							.read(),
					..Default::default()
				}
			})
			.collect(),
		has_more: page < data.get("max_page").as_int().unwrap() as i32,
	})
}

pub fn parse_manga_details(data: ObjectRef) -> Result<Manga> {
	let id = data.get("id").as_int().unwrap().to_string();
	let cover = resolve_image_url(&data.get("cover_mobile_url").as_string().unwrap().read());
	let title = parse_title(&data.get("titles").as_array().unwrap());
	let author = get_list_name(&data.get("authors").as_array().unwrap()).join(", ");
	let description = data.get("full_description").as_string().unwrap().read();
	let url = format!("{BASE_URL}/mangas/${id}");
	let categories = get_list_name(&data.get("tags").as_array().unwrap());
	let status = MangaStatus::Unknown;
	let nsfw = if data.get("is_nsfw").as_bool().unwrap_or(false) {
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

pub fn parse_chapter_list(data: ArrayRef, manga_id: String) -> Result<Vec<Chapter>> {
	let total = data.len() as f32;

	Ok(data
		.enumerate()
		.map(|(idx, chapter_ref)| {
			let chapter_data = chapter_ref.as_object().unwrap();

			let id = chapter_data.get("id").as_int().unwrap().to_string();
			let title = chapter_data
				.get("name")
				.as_string()
				.unwrap_or("".into())
				.read();

			let number = chapter_data.get("number").as_string().unwrap().read();
			let (volume, chapter) = if number.contains("vol.") {
				// number sample: Bonus vol. 1
				// chapter = -1.0
				let vol_value = number
					.split("vol.")
					.nth(1)
					.map(|s| s.trim())
					.and_then(|s| s.parse::<f32>().ok())
					.unwrap_or(-1.0);

				(vol_value, total - idx as f32)
			} else {
				(
					-1.0,
					chapter_data
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
						.unwrap_or(total - idx as f32),
				)
			};

			Chapter {
				id: id.clone(),
				title,
				volume,
				chapter,
				date_updated: chapter_data
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
				lang: "vi".to_string(),
				..Default::default()
			}
		})
		.collect())
}

pub fn parse_page_list(data: ObjectRef) -> Result<Vec<Page>> {
	let chapters = data.get("pages").as_array().unwrap();

	if chapters.clone().count() == 0 {
		return Err(AidokuError {
			reason: AidokuErrorKind::DefaultNotFound,
		});
	};

	let key = data.get("key").as_string().unwrap().read();
	let pages = chapters
		.map(|chapter_ref| {
			let chapter = chapter_ref.as_object().unwrap();

			let index = chapter.get("order").as_int().unwrap() as i32;
			let url = resolve_image_url(
				&decrypt_data_with_wrapped_key(
					key.clone(),
					chapter.get("path").as_string().unwrap().read(),
				)
				.unwrap(),
			);

			Page {
				index,
				url,
				..Default::default()
			}
		})
		.collect();

	Ok(pages)
}
