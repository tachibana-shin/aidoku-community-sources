use aidoku::std::{ArrayRef, String, Vec};
use alloc::{borrow::ToOwned, string::ToString};
use url::Url;

pub fn resolve_image_url(url: &str) -> String {
	// If it's already an absolute path starting with '/', return directly
	if url.starts_with('/') {
		return url.to_string();
	}

	// Base URL for relative paths
	let base = Url::parse("https://p21-ad-sg.ibyteimg.com/obj/").unwrap();

	// Resolve the relative URL using `Url::join`
	match base.join(url) {
		Ok(resolved) => resolved.to_string(),
		Err(_) => url.to_string(), // fallback
	}
}

pub fn parse_title(arr: &ArrayRef) -> String {
	// primary == true のタイトルを探す
	for entry in arr.to_owned() {
		let entry = entry.as_object().unwrap();
		if entry.get("primary").as_bool().unwrap() {
			return entry.get("name").as_string().unwrap().read();
		}
	}

	// primary が存在しない場合は最初のタイトルを使う
	arr.get(0)
		.as_object()
		.unwrap()
		.get("name")
		.as_string()
		.unwrap()
		.read()
		.to_string()
}

pub fn get_list_name(arr: &ArrayRef) -> Vec<String> {
	arr.clone()
		.map(|tag_ref| {
			tag_ref
				.as_object()
				.and_then(|tag| tag.get("name").as_string())
				.map(|s| s.read())
				.unwrap_or_default()
		})
		.collect::<Vec<_>>()
}
