#![no_std]
#![allow(static_mut_refs)]
extern crate alloc;
mod parser;
mod search;

use aidoku::{
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page,
	error::{AidokuError, Result},
	prelude::*,
	std::{String, Vec, html::Node, net::Request},
};
use alloc::string::ToString;
use parser::{parse_chapter_list, parse_manga_details, parse_page_list, parse_search_page};
use search::get_search_url;

pub static BASE_URL: &str = "https://docln.sbs";

use core::cell::UnsafeCell;

struct Cache {
	id: Option<String>,
	data: Option<Vec<u8>>,
}

struct GlobalCache(UnsafeCell<Cache>);
unsafe impl Sync for GlobalCache {}

static CACHE: GlobalCache = GlobalCache(UnsafeCell::new(Cache {
	id: None,
	data: None,
}));

fn req_with_cache(url: String) -> Node {
	let cache = unsafe { &mut *CACHE.0.get() };

	if let Some(cached_id) = &cache.id {
		if *cached_id == url {
			if let Some(data) = &cache.data {
				return Node::new(data).expect("Invalid cached node");
			}
		}
	}

	let req = Request::get(url.clone()).header("Referer", BASE_URL);
	let html_data = req.data();

	cache.id = Some(url);
	cache.data = Some(html_data.clone());

	Node::new(&html_data).expect("Invalid node")
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let search_url = get_search_url(filters, page);
	let req = Request::get(&search_url).header("Referer", BASE_URL);

	parse_search_page(req.html()?)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let page_url = match listing.name.as_str() {
		"Sáng tác" => "sang-tac",
		"AI dịch" => "ai-dich",
		"Danh sách" => "danh-sach",
		_ => {
			return Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			});
		}
	};
	let req =
		Request::get(format!("{BASE_URL}/{page_url}?page={page}")).header("Referer", BASE_URL);
	parse_search_page(req.html()?)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/{}", BASE_URL, id);

	parse_manga_details(url.clone(), id, req_with_cache(url))
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let id = format!("{}/{}", BASE_URL, id);

	parse_chapter_list(req_with_cache(id.clone()))
}

#[get_page_list]
fn get_page_list(path: String, name: String) -> Result<Vec<Page>> {
	let url = format!("{}/{}", BASE_URL, path);

	let document = req_with_cache(url);

	parse_page_list(document, &name, BASE_URL)
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
	if manga_or_chapter_id.contains("truyen") || manga_or_chapter_id.contains("ai-dich") {
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
