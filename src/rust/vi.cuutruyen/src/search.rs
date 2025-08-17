use crate::BASE_URL;
use aidoku::{
	Filter, FilterType,
	prelude::format,
	std::{String, Vec},
};
use alloc::string::ToString;

pub fn get_search_url(filters: Vec<Filter>, page: i32) -> String {
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(val) = filter.value.as_string() {
					let title = val.read();

					return format!(
						"{BASE_URL}/api/v2/mangas/search?q={title}&page={page}&per_page=30",
				
					);
				}
			}
			FilterType::Genre => {
				return format!(
					"{BASE_URL}/api/v2/tags/{}?page={}&per_page=30",
					&filter.object.get("id").as_string().unwrap(),
					&page.to_string()
				);
			}
			_ => {}
		}
	}

	return BASE_URL.to_string();
}
