use crate::BASE_URL;
use aidoku::{
	helpers::uri::QueryParameters,
	prelude::format,
	std::{String, Vec},
	Filter, FilterType,
};

pub fn get_search_url(filters: Vec<Filter>, page: i32) -> String {
	let mut qs = QueryParameters::new();
	qs.push("post_type", Some("wp-manga"));

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(title) = filter.value.as_string() {
					qs.push("s", Some(&title.read()));
				}
			}
			_ => continue,
		}
	}
	format!("{BASE_URL}/page/{page}/?{qs}")
}
