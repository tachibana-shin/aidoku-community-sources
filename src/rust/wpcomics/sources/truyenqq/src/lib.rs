#![no_std]
use aidoku::{
	error::Result,
	prelude::*,
	std::{defaults::defaults_get, net::HttpMethod, net::Request, String, StringRef, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use wpcomics_template::helper::extract_f32_from_string;
use wpcomics_template::{helper::urlencode, template::WPComicsSource};

fn get_base_url() -> Result<String> {
	defaults_get("baseURL")?
		.as_string()
		.map(|v| String::from(v.read().trim_end_matches('/')))
}
fn get_proxy_url() -> Result<String> {
	defaults_get("proxy")?
		.as_string()
		.map(|v| String::from(v.read().trim_end_matches('/')))
}
fn get_visit_read_id() -> Result<String> {
	defaults_get("visitReadId")?
		.as_string()
		.map(|v| String::from(v.read()))
}

const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

fn get_url_with_proxy(url: &str) -> String {
	// if proxy == null -> skip
	match get_proxy_url() {
		Ok(proxy) => {
			if proxy.is_empty() {
				String::from(url)
			} else {
				format!("{}?url={}", proxy, url)
			}
		}
		Err(_) => String::from(url),
	}
}

fn get_instance() -> WPComicsSource {
	WPComicsSource {
		base_url: String::from(get_base_url().unwrap_or_default()),
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

		page_url_transformer: |url| url,
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
						return get_url_with_proxy(&format!(
							"{}/tim-kiem/trang-{page}.html?q={title}",
							get_base_url().unwrap_or_default()
						));
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
		get_url_with_proxy(&format!(
			"{}/tim-kiem-nang-cao.html?category={}&notcategory={}{}",
			get_base_url().unwrap_or_default(),
			included_tags.join(","),
			excluded_tags.join(","),
			query
		))
	}
	let cookie_str = format!("visit-read={}", get_visit_read_id().unwrap_or_default());
	let headers = &[("Cookie", cookie_str.as_str())];
	get_instance().get_manga_list(get_search_url(filters, page), Some(headers))
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	get_instance().get_manga_listing(listing, page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	get_instance().get_manga_details(get_url_with_proxy(&id))
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	// for i in 1..=5 {
	//     let chapter_id = format!("fake-chap-{}", i);
	//     chapters.push(Chapter {
	//         id: chapter_id.clone(),
	//         title: get_url_with_proxy(&id),
	//         volume: -1.0,
	//         chapter: i as f32,
	//         date_updated: 0.0,
	//         url: format!("https://example.com/manga/{}/chapter/{}", id, i),
	//         lang: String::from("en"),
	//         ..Default::default()
	//     });
	// }

	let mut req = Request::new(&get_url_with_proxy(&id), HttpMethod::Get);
	req = req.header("User-Agent", USER_AGENT);

	let html = req.html()?;

	// // =================== fork from template.rs ====================
	let title_untrimmed = (get_instance().manga_details_title_transformer)(
		html.select("div.book_other h1[itemprop=name]")
			.text()
			.read(),
	);
	let title = title_untrimmed.trim();

	for chapter in html.select("div.works-chapter-item").array() {
		let chapter_node = chapter.as_node().expect("node array");
		let mut chapter_url = chapter_node.select("div.name-chap a").attr("href").read();

		if !chapter_url.contains("http://") && !chapter_url.contains("https://") {
			chapter_url = format!(
				"{}{}{}",
				String::from(get_base_url().unwrap_or_default()),
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
		let date_updated =
			(get_instance().time_converter)(chapter_node.select("div.time-chap").text().read());
		chapters.push(Chapter {
			id: get_url_with_proxy(&chapter_id),
			title: String::from(chapter_title.trim()),
			volume,
			chapter,
			date_updated,
			url: get_url_with_proxy(&chapter_url),
			lang: String::from("en"),
			..Default::default()
		});
	}
	// =====================================================
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let mut req = Request::new(&get_url_with_proxy(&chapter_id), HttpMethod::Get);
	req = req.header("User-Agent", USER_AGENT);

	let html = req.html()?;

	for (at, page) in html.select("div.page-chapter > img").array().enumerate() {
		let page_node = page.as_node().expect("node array");

		let mut page_url = page_node.attr("data-original").read();
		if page_url.is_empty() {
			page_url = page_node.attr("data-cdn").read();
		}
		if page_url.is_empty() {
			page_url = page_node.attr("src").read();
		}

		if !page_url.starts_with("http") {
			page_url = String::from("https:") + &page_url;
		}
		pages.push(Page {
			index: at as i32,
			url: page_url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	get_instance().modify_image_request(request)
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	get_instance().handle_url(get_url_with_proxy(&url))
}
