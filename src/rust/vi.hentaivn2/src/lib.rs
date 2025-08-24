#![no_std]
extern crate alloc;
mod parser;
mod search;

use aidoku::{
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
	error::{AidokuError, Result},
	helpers::uri::encode_uri,
	prelude::*,
	std::{String, Vec, net::Request},
};
use alloc::string::ToString;
use parser::{
	parse_chapter_list, parse_manga_details, parse_page_list,
	parse_search_page,
};
use search::get_search_url;

pub static BASE_URL: &str = "https://hentaivn.su";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let search_url = get_search_url(filters, page);
	let req = Request::get(&search_url).header("Referer", BASE_URL);
	parse_search_page(req.json()?.as_object()?)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let page_url = match listing.name.as_str() {
		"Không che" => "/api/library/genre/87?sort=updated_at",
		"Full color" => "/api/library/genre/136?sort=updated_at",
		_ => {
			return Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			});
		}
	};
	let req =
		Request::get(format!("{BASE_URL}/{page_url}&page={page}")).header("Referer", BASE_URL);
	parse_search_page(req.json()?.as_object()?)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = encode_uri(format!("{BASE_URL}/api/manga/{id}"));
	let req = Request::get(url).header("Referer", BASE_URL);
	parse_manga_details(req.json()?.as_object()?)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = encode_uri(format!("{BASE_URL}/api/manga/{}/chapters", id));
	let req = Request::get(url).header("Referer", BASE_URL);
	parse_chapter_list(req.json()?.as_array()?, id)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let url = encode_uri(format!("{BASE_URL}/api/chapter/{}", id));
	let req = Request::get(url).header("Referer", BASE_URL);

	parse_page_list(req.json()?.as_object()?)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let manga_or_chapter_id = url
		.split('/')
		.last()
		.expect("handle_url expected last element");
	if manga_or_chapter_id.contains("manga") {
		Ok(DeepLink {
			manga: get_manga_details(manga_or_chapter_id.to_string()).ok(),
			chapter: None,
		})
	} else {
		Err(AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		})
	}
}
