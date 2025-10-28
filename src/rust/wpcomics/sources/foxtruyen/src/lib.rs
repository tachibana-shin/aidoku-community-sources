#![no_std]
pub mod helper;
use crate::helper::*;
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter,
	FilterType, Listing, Manga, MangaPageResult, MangaViewer, Page,
};
use wpcomics_template::{helper::urlencode, template, template::WPComicsSource};

const BASE_URL: &str = "https://foxtruyen.com";

fn get_instance() -> WPComicsSource {
	WPComicsSource {
		base_url: String::from(BASE_URL),
		cookie: Some("type_book=1"),
		next_page: ".page_redirect > a:nth-last-child(2) > p:not(.active)",
		viewer: MangaViewer::Rtl,
		status_mapping: status_map,
		manga_viewer_page_attr: "src",
		manga_cell: ".item_home",
		manga_cell_title: ".book_name",
		manga_cell_url: ".book_name",
		manga_cell_image: "img",
		manga_cell_image_attr: "data-src",

		manga_details_title: ".title_tale h1",
		manga_details_cover: ".thumbblock img",
		manga_details_chapters: ".item_chap",
		chapter_anchor_selector: "a",
		chapter_date_selector: "em",

		manga_parse_id: |url| {
			String::from(
				url.split("truyen-tranh/")
					.nth(1)
					.and_then(|s| s.split('/').next())
					.unwrap_or_default()
					.trim_end_matches(".html"),
			)
		},
		chapter_parse_id: |url| {
			String::from(
				url.trim_end_matches('/')
					.rsplit("-chap-")
					.next()
					.unwrap()
					.trim_end_matches(".html"),
			)
		},
		time_converter: |ago| {
			aidoku::std::StringRef::from(ago)
				.0
				.as_date("dd/MM/yyyy", None, Some("Asia/Ho_Chi_Minh"))
				.unwrap_or(-1.0)
		},
		manga_viewer_page: ".content_detail_manga img",
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut title: String = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = urlencode(filter.value.as_string()?.read());
			}
			_ => {}
		}
	}
	let instance = get_instance();
	instance.get_manga_list(get_search_url(instance.base_url.clone(), title, page), None)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	get_instance().get_manga_details(format!("{}/truyen-tranh/{}", BASE_URL, id))
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	get_instance().get_chapter_list(format!("{}/truyen-tranh/{}", BASE_URL, id))
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	get_instance().get_page_list(format!(
		"{}/truyen-tranh/{}-chap-{}",
		BASE_URL, manga_id, chapter_id
	))
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	template::modify_image_request(
		String::from(BASE_URL),
		String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.54 Safari/537.36 Edg/101.0.1210.39"),
		request,
	)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
