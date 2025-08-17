#![no_std]
#![allow(static_mut_refs)]
extern crate alloc;
mod parser;
mod search;

use aidoku::{
	error::{AidokuError, Result}, prelude::*, std::{net::Request, String, Vec}, Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, Page
};
use alloc::string::ToString;
use parser::{parse_chapter_list, parse_manga_details, parse_page_list, parse_search_page};
use search::get_search_url;

pub static BASE_URL: &str = "https://cuutruyen5c844.site";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let search_url = get_search_url(filters, page);

	let req = Request::get(&search_url).header("Referer", BASE_URL);
	
	let data = req.json()?.as_object()?;

	parse_search_page(data, page)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let page_url = match listing.name.as_str() {
		"Mới cập nhật" => "recently_updated",
		_ => {
			return Err(AidokuError {
				reason: aidoku::error::AidokuErrorKind::Unimplemented,
			});
		}
	};
	let req = Request::get(format!(
		"{BASE_URL}/api/v2/mangas/{page_url}?page={page}&per_page=30"
	))
	.header("Referer", BASE_URL);
	parse_search_page(req.json()?.as_object()?, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let req = Request::get(format!("{BASE_URL}/api/v2/mangas/{id}")).header("Referer", BASE_URL);
	parse_manga_details(req.json()?.as_object()?)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let req = Request::get(format!("{BASE_URL}/api/v2/mangas/{id}/chapters")).header("Referer", BASE_URL);
	parse_chapter_list(req.json()?.as_object()?, id)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let req = Request::get(format!("{BASE_URL}/api/v2/chapters/{id}")).header("Referer", BASE_URL);

	parse_page_list(req.json()?.as_object()?)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let segments: Vec<&str> = url.split('/').collect();

    if let Some(pos) = segments.iter().position(|&s| s == "mangas") {
        if let Some(manga_id) = segments.get(pos + 1) {
            return Ok(DeepLink {
                manga: get_manga_details(manga_id.to_string()).ok(),
                chapter: None,
            });
        }
    }

    Err(AidokuError {
        reason: aidoku::error::AidokuErrorKind::Unimplemented,
    })
}
