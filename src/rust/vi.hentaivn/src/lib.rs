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
	parse_chapter_list, parse_manga_details, parse_new_or_complete_page, parse_page_list,
	parse_search_page,
};
use search::get_search_url;

pub static BASE_URL: &str = "https://hentaivn.cx";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let search_url = get_search_url(filters, page);
	println!("url == {}", search_url);
	let req = Request::get(&search_url).header("Referer", BASE_URL);
	parse_search_page(req.html()?)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let page_url = match listing.name.as_str() {
		"Chương mới" => "chap-moi.html",
		"Đã hoàn thành" => "da-hoan-thanh.html",
		_ => {
			return Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			});
		}
	};
	let req =
		Request::get(format!("{BASE_URL}/{page_url}?page={page}")).header("Referer", BASE_URL);
	parse_new_or_complete_page(req.html()?)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = encode_uri(format!("{BASE_URL}/{id}"));
	let req = Request::get(url).header("Referer", BASE_URL);
	parse_manga_details(id, req.html()?)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let req = Request::get(id).header("Referer", BASE_URL);
	parse_chapter_list(req.html()?)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let req = Request::get(id).header("Referer", BASE_URL);

	parse_page_list(req.html()?, Some(".wp-manga-chapter-img"))
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
	if manga_or_chapter_id.contains("doc-truyen") {
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
