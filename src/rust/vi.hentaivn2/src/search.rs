use crate::BASE_URL;
use aidoku::{
	helpers::uri::QueryParameters,
	prelude::format,
	std::{String, Vec},
	Filter, FilterType,
};
use alloc::string::ToString;

pub fn get_search_url(filters: Vec<Filter>, page: i32) -> String {
	let mut qs = QueryParameters::new();
	qs.push("sort", Some("updated_at"));
	qs.push("page", Some(&page.to_string()));

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(title) = filter.value.as_string() {
					qs.push("q", Some(&title.read()));
				}
			},
			FilterType::Genre => {
				if let Ok(genre) = filter.value.as_string() {
					return format!("{BASE_URL}/api/library/genre/{}?sort=updated_at&page={page}", genre.read());
				}
			}
			_ => continue,
		}
	}
	format!("{BASE_URL}/api/library/search/?{qs}")
}
