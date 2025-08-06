#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{defaults::defaults_get, net::Request, String, StringRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use wpcomics_template::{helper::urlencode, template::WPComicsSource};
use wpcomics_template::helper::{extract_f32_from_string};

const BASE_URL: &str = "https://truyenqqgo.com";
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

fn get_instance() -> WPComicsSource {
	WPComicsSource {
		base_url: String::from(BASE_URL),
		viewer: MangaViewer::Rtl,
		listing_mapping: |listing| {
			String::from(match listing.as_str() {
				"Truyện con gái" => "truyen-con-gai",
				"Truyện con trai" => "truyen-con-trai",
				_ => "",
			})
		},
		status_mapping: |status| match status.trim() {
			"Đang Cập Nhật" => MangaStatus::Ongoing,
			"Hoàn Thành" => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		},
		time_converter: |ago| {
			StringRef::from(ago)
				.0
				.as_date("dd/MM/yyyy", None, Some("Asia/Ho_Chi_Minh"))
				.unwrap_or(-1.0)
		},

		next_page: ".page_redirect > a:nth-last-child(2) > p:not(.active)",
		manga_cell: "ul.grid li",
		manga_cell_title: ".book_info .qtip a",
		manga_cell_url: ".book_info .qtip a",
		manga_cell_image: ".book_avatar img",
		manga_cell_image_attr: "abs:src",

		manga_listing_pagination: "/trang-",
		manga_listing_extension: ".html",

		manga_details_title: "div.book_other h1[itemprop=name]",
		manga_details_cover: "div.book_avatar img",
		manga_details_author: "li.author.row p.col-xs-9",
		manga_details_description: "div.story-detail-info.detail-content",
		manga_details_tags: "ul.list01 > li",
		manga_details_tags_splitter: "",
		manga_details_status: "li.status.row p.col-xs-9",
		manga_details_chapters: "div.works-chapter-item",

		chapter_skip_first: false,
		chapter_anchor_selector: "div.name-chap a",
		chapter_date_selector: "div.time-chap",

		page_url_transformer: |url| {
			url
		},
		vinahost_protection: true,
		user_agent: Some(USER_AGENT),
		..Default::default()
	}
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	fn get_search_url(filters: Vec<Filter>, page: i32) -> String {
		let mut excluded_tags: Vec<String> = Vec::new();
		let mut included_tags: Vec<String> = Vec::new();
		let mut query = String::new();
		for filter in filters {
			match filter.kind {
				FilterType::Title => {
					let title = urlencode(
						filter
							.value
							.as_string()
							.unwrap_or_else(|_| StringRef::from(""))
							.read(),
					);
					if !title.is_empty() {
						return format!("{BASE_URL}/tim-kiem/trang-{page}.html?q={title}");
					}
				}
				FilterType::Genre => {
					let genre = filter
						.object
						.get("id")
						.as_string()
						.unwrap_or_else(|_| StringRef::from(""))
						.read();
					if genre.is_empty() {
						continue;
					}
					match filter.value.as_int().unwrap_or(-1) {
						0 => excluded_tags.push(genre),
						1 => included_tags.push(genre),
						_ => continue,
					}
				}
				_ => match filter.name.as_str() {
					"Tình trạng" => {
						let mut status = filter.value.as_int().unwrap_or(-1);
						if status == 0 {
							status = -1
						}
						query.push_str("&status=");
						query.push_str(format!("{}", status).as_str());
					}
					"Quốc gia" => {
						let country = filter.value.as_int().unwrap_or(-1);
						if country >= 0 {
							query.push_str("&country=");
							query.push_str(format!("{}", country).as_str());
						}
					}
					"Số lượng chapter" => {
						let minchapter = match filter.value.as_int().unwrap_or(-1) {
							0 => "0",
							1 => "50",
							2 => "100",
							3 => "200",
							4 => "300",
							5 => "400",
							6 => "500",
							_ => continue,
						};
						query.push_str("&minchapter=");
						query.push_str(minchapter);
					}
					"Sắp xếp theo" => {
						let sort = filter.value.as_int().unwrap_or(-1);
						if sort >= 0 {
							query.push_str("&sort=");
							query.push_str(format!("{}", sort).as_str());
						}
					}
					_ => continue,
				},
			}
		}
		format!(
			"{BASE_URL}/tim-kiem-nang-cao.html?category={}&notcategory={}{}",
			included_tags.join(","),
			excluded_tags.join(","),
			query
		)
	}
	let headers = &[("Cookie", "visit-read=6806034e0db74-6806034e0db75")];
	get_instance().get_manga_list(get_search_url(filters, page), Some(headers))
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	get_instance().get_manga_details(id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let html = get_instance()
		.request_vinahost(&format!("{}", id), None)
		.html()?;
	// =================== fork from template.rs ====================
	let title_untrimmed = (get_instance().manga_details_title_transformer)(
		html.select("div.book_other h1[itemprop=name]").text().read(),
	);
	let title = title_untrimmed.trim();

	let mut chapters: Vec<Chapter> = Vec::new();

	for chapter in html.select("div.works-chapter-item").array() {
		let chapter_node = chapter.as_node().expect("node array");
		let mut chapter_url = chapter_node.select("div.name-chap a").attr("href").read();

		if !chapter_url.contains("http://") && !chapter_url.contains("https://") {
			chapter_url = format!(
				"{}{}{}",
				String::from(BASE_URL),
				if chapter_url.starts_with("/") {
					""
				} else {
					"/"
				},
				chapter_url
			);
		}
		let chapter_id = chapter_url.clone();
		let mut chapter_title = chapter_node.select("div.name-chap a").text().read();
		let numbers = extract_f32_from_string(String::from(title), String::from(&chapter_title));
		let (volume, chapter) =
			if numbers.len() > 1 && chapter_title.to_ascii_lowercase().contains("vol") {
				(numbers[0], numbers[1])
			} else if !numbers.is_empty() {
				(-1.0, numbers[0])
			} else {
				(-1.0, -1.0)
			};
		if chapter >= 0.0 {
			let splitter = format!(" {}", chapter);
			let splitter2 = format!("#{}", chapter);
			if chapter_title.contains(&splitter) {
				let split = chapter_title.splitn(2, &splitter).collect::<Vec<&str>>();
				chapter_title =
					String::from(split[1]).replacen(|char| char == ':' || char == '-', "", 1);
			} else if chapter_title.contains(&splitter2) {
				let split = chapter_title.splitn(2, &splitter2).collect::<Vec<&str>>();
				chapter_title =
					String::from(split[1]).replacen(|char| char == ':' || char == '-', "", 1);
			}
		}
		let date_updated = (get_instance().time_converter)(
			chapter_node
				.select("div.time-chap")
				.text()
				.read(),
		);
		chapters.push(Chapter {
			id: chapter_id,
			title: String::from(chapter_title.trim()),
			volume,
			chapter: -1.0,
			date_updated,
			url: chapter_url,
			lang: String::from("en"),
			..Default::default()
		});
	}
	// =====================================================
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	get_instance().get_page_list(chapter_id)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(url)
}
