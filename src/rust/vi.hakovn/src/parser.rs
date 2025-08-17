use alloc::string::ToString;

use aidoku::{
	Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::{format, println},
	std::{String, Vec, html::Node, net::Request},
};

use crate::BASE_URL;
fn absolute_url(url: String, base_url: String) -> String {
	if url.starts_with("http://") || url.starts_with("https://") {
		url
	} else if url.starts_with('/') {
		if let Some(pos) = base_url.find("://") {
			if let Some(slash_pos) = base_url[pos + 3..].find('/') {
				let domain = &base_url[..pos + 3 + slash_pos + pos + 3];
				format!("{}{}", domain, url)
			} else {
				format!("{}{}", base_url, url)
			}
		} else {
			format!("{}{}", base_url, url)
		}
	} else {
		let mut new_base = base_url;
		if !new_base.ends_with('/') {
			new_base.push('/');
		}
		new_base.push_str(&url);
		new_base
	}
}

pub fn parse_search_page(document: Node) -> Result<MangaPageResult> {
	let nodes = document.select(".thumb-item-flow");
	let elems = nodes.array();

	let mut manga: Vec<Manga> = Vec::with_capacity(elems.len());

	for (_id, elem) in elems.enumerate() {
		if let Ok(node) = elem.as_node() {
			let url_elem = node.select(".series-title a").first();
			let id = absolute_url(url_elem.attr("href").read(), BASE_URL.to_string());

			manga.push(Manga {
				id,
				cover: node.select(".img-in-ratio").attr("data-bg").read(),
				title: url_elem.text().read(),
				..Default::default()
			})
		}
	}

	let last_page = document.select(".pagination > a");
	Ok(MangaPageResult {
		manga,
		has_more: !last_page.array().is_empty()
			&& last_page.last().text().read().contains("Cuối")
			&& !last_page.first().has_class("disabled"),
	})
}

pub fn parse_manga_details(id: String, document: Node) -> Result<Manga> {
	let title_elem = document.select(".series-name a");
	let title = title_elem.text().read().to_string();
	let url = id.clone();

	let author_elem = document.select(".info-name:contains(Tác giả) + span");
	let author = author_elem.text().read().trim().to_string();

	let cover_elem = document.select(".series-cover .img-in-ratio");
	let cover = cover_elem
		.attr("style")
		.read()
		.trim()
		.strip_prefix("background-image: url('")
		.and_then(|s| s.strip_suffix("')"))
		.unwrap_or_default()
		.to_string();

	let status_elem = document.select("info-name:contains(Tình Trạng) + span");
	let status = if status_elem.text().read() == "Đã hoàn thành" {
		MangaStatus::Completed
	} else {
		MangaStatus::Ongoing
	};

	let nsfw = MangaContentRating::Suggestive;
	let viewer = MangaViewer::Ltr;
	let category_elems = document.select("a.series-gerne-item");
	let categories = category_elems
		.array()
		.filter_map(|elem| {
			if let Ok(node) = elem.as_node() {
				let category = node.text().read();
				Some(category)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	let description = format!(
		"{} - {}",
		document.select(".summary-wrapper").html().read(),
		document.select(".series-summary").html().read()
	);

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
		viewer,
		..Default::default()
	})
}

pub fn parse_chapter_list(document: Node, root_url: String) -> Result<Vec<Chapter>> {
	let volumes = document.select(".volume-list").array();
	let mut chapters = Vec::with_capacity(volumes.len());

	for (idx, volume) in volumes.rev().enumerate() {
		if let Ok(node) = volume.as_node() {
			let url_elem = node.select(".chapter-name a").first();

			let title_raw = node.select(".sect-title").first().text().read();

			let id = format!(
				"{{\"root\": \"{}\", \"name\": \"{}\"}}",
				root_url, title_raw
			);

			let mut chapter = idx as f32;
			let title_parts = title_raw
				.split(|v| v == '-' || v == ':')
				.collect::<Vec<_>>();

			let title = if title_parts.len() > 1 {
				title_parts[1].trim().to_string()
			} else {
				title_raw.clone()
			};

			if title_parts[0].contains("Tập") {
				let chapter_raw = title_parts[0]
					.split(char::is_whitespace)
					.last()
					.expect("chapter number");
				chapter = chapter_raw.parse::<f32>().unwrap_or(idx as f32);
			}

			chapters.push(Chapter {
				id,
				chapter,
				title,
				date_updated: 0.0,
				url: absolute_url(url_elem.attr("href").read(), BASE_URL.to_string()),
				..Default::default()
			})
		}
	}

	Ok(chapters)
}

fn extract_url_from_style(style: &str) -> Option<String> {
	let start = style.find("url(")?;
	let end = style[start..].find(')')? + start;

	let mut url = &style[start + 4..end];

	if url.starts_with('\'') && url.ends_with('\'') || url.starts_with('\"') && url.ends_with('\"')
	{
		url = &url[1..url.len() - 1];
	}

	Some(url.to_string())
}

pub fn parse_page_list(document: Node, selector: &str, base_url: &str) -> Result<Vec<Page>> {
	println!("{}", selector.to_string());

	let page_elems = document.select(".volume-list").array().enumerate();
	let mut chapters: Option<Node> = None;
	let mut cover_url: Option<String> = None;
	for (_index, volume) in page_elems {
		if let Ok(node) = volume.as_node() {
			let header_text = node.select("header").first().text().read();

			println!("title = '{}' , selector = '{}'", header_text, selector);
			if header_text.contains(selector) {
				cover_url = extract_url_from_style(
					&node.select(".content.img-in-ratio").attr("style").read(),
				);
				chapters = Some(node.select(".list-chapters > li a"));
				break;
			}
		}
	}

	if chapters.is_none() {
		return Err(AidokuError {
			reason: AidokuErrorKind::DefaultNotFound,
		});
	};

	let mut pages = Vec::new();

	pages.push(Page {
		text: "novel".to_string(),
		index: -1,
		url: cover_url.unwrap(),
		base64: format!(
			"{} - {}",
			document.select(".series-name a").text().read(),
			selector
		),
	});

	for (index, elem) in chapters.unwrap().array().enumerate() {
		if let Ok(anchor) = elem.as_node() {
			let url = absolute_url(anchor.attr("href").read(), BASE_URL.to_string());

        let req = Request::get(&url).header("Referer", base_url);

        // Try-catch kiểu Rust cho request
        let document = match req.html() {
            Ok(doc) => doc,
            Err(e) => {
                println!("[ERROR] Failed to fetch HTML for {}: {:?}", url, e);
                continue; // bỏ qua và sang chapter tiếp theo
            }
        };

			let text = document
				.select("#chapter-content")
				.first()
				.html()
				.read()
				.replace(r#"src="/"#, &format!(r#"src="{}/"#, base_url))
				.replace(r#"src='/\"#, &format!(r#"src='{}/"#, base_url))
				.replace(r#"src=/"#, &format!(r#"src={}")"#, base_url))
				.replace(
					&document
						.select("#chapter-content img.d-none")
						.first()
						.html()
						.read()
						.to_string(),
					"",
				)
				.replace(
					&document
						.select("#chapter-content img.d-md-none")
						.first()
						.html()
						.read()
						.to_string(),
					"",
				);

			let base64 = anchor.text().read();

			pages.push(Page {
				index: index as i32,
				text,
				base64,
				url,
				..Default::default()
			});
		}
	}

	println!("pagess  = {}", pages.len());

	Ok(pages)
}
