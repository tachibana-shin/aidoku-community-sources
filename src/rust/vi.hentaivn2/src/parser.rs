use alloc::string::ToString;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result}, prelude::format, std::{ArrayRef, ObjectRef, String, Vec}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, Page
};
use time::format_description::well_known::Rfc3339;

use crate::BASE_URL;

pub fn parse_search_page(data: ObjectRef) -> Result<MangaPageResult> {
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
					cover: format!(
						"{BASE_URL}{}",
						item.get("coverUrl").as_string().unwrap().read()
					),
					title: item.get("title").as_string().unwrap().read(),
					author: item.get("authors").as_string().unwrap_or_default().read(),
					description: item
						.get("alternativeTitles")
						.as_string()
						.unwrap_or_default()
						.read(),
					..Default::default()
				}
			})
			.collect(),
		has_more: data.get("page").as_int().unwrap()
			< data.get("total").as_int().unwrap() / data.get("limit").as_int().unwrap(),
	})
}

pub fn parse_manga_details(meta: ObjectRef) -> Result<Manga> {
	let id = meta.get("id").as_int().unwrap().to_string();
	let cover = format!(
		"{BASE_URL}{}",
		meta.get("coverUrl").as_string().unwrap().read()
	);
	let title = format!(
		"{} - {}",
		meta.get("title").as_string().unwrap().read(),
		meta.get("alternativeTitles")
			.as_array()
			.unwrap()
			.map(|title_ref| title_ref.as_string().unwrap().read())
			.collect::<Vec<_>>()
			.join(", ")
	);
	let author = meta
		.get("authors")
		.as_array()
		.unwrap()
		.map(|author_ref| {
			let author = author_ref.as_object().unwrap();
			author.get("name").as_string().unwrap().read()
		})
		.collect::<Vec<_>>()        // -> Vec<&str>
    .join(", ");
	let description = meta.get("description").as_string().unwrap().read();
	let url = format!("{BASE_URL}/manga/${id}");
	let categories = meta
		.get("genres")
		.as_array()
		.unwrap()
		.map(|tag_ref| {
			let tag = tag_ref.as_object().unwrap();

			tag.get("name").as_string().unwrap().read()
		})
		.collect();
	let status = MangaStatus::Unknown;
	let nsfw = MangaContentRating::Nsfw;

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
	let total = data.clone().len() as f32;

	Ok(data
		.enumerate()
		.map(|(idx, chapter_ref)| {
			let chapter_data = chapter_ref.as_object().unwrap();

			let id = chapter_data.get("id").as_int().unwrap().to_string();
			let title = chapter_data
				.get("title")
				.as_string()
				.unwrap_or("".into())
				.read();

			let number = chapter_data.get("readOrder").as_int().unwrap().to_string();
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
					.get("createdAt")
					.as_string()
					.and_then(|s| {
						time::OffsetDateTime::parse(&s.read(), &Rfc3339).map_err(|_| AidokuError {
							reason: AidokuErrorKind::JsonParseError,
						})
					})
					.map(|dt| dt.unix_timestamp() as f64)
					.unwrap_or(-1.0),
				url: format!("{BASE_URL}/manga/{manga_id}/{id}"),
				lang: "en".to_string(),
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

	let pages = chapters
		.enumerate()
		.map(|(idx, chapter_ref)| {
			let chapter = chapter_ref.as_string().unwrap();

			let index = idx as i32;
			let url = format!("{BASE_URL}{}", chapter.read());

			Page {
				index,
				url,
				..Default::default()
			}
		})
		.collect();

	Ok(pages)
}
