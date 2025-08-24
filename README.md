# Aidoku Sources
This repository hosts unofficial, community-made sources that are installable through the Aidoku application.

> [!WARNING]
> The Light Novel sources are only supported for [Rakuyomi Fork version](https://github.com/tachibana-shin/rakuyomi)

## Usage
On a device with Aidoku installed, you can open [this link](https://aidoku.app/add-source-list/?url=https://raw.githubusercontent.com/tachibana-shin/aidoku-community-sources/gh-pages/) to add the source list directly to the app.

Otherwise, navigate to the settings tab, and under the source lists page add `https://raw.githubusercontent.com/tachibana-shin/aidoku-community-sources/gh-pages/`.

If a source is not working, or you want to request a source that isn't available in this source list, feel free to [create a new issue](https://github.com/tachibana-shin/aidoku-community-sources/issues).

## Light Novel Source
Light novel sources must be designed according to the following rules:
- `get_chapter_list` - Instead of these function chapter requires you to return the volume
```rust
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
```
- `get_page_list` - This function requires you the first item to be the following form:
The first page:
```rust
Page {
	text: "novel".to_string(), // const
	index: -1, // const
	url: cover_url.unwrap(), // String is url cover volume or ""
	base64: format!(
		"{} - {}",
		document.select(".series-name a").text().read(),
		selector
	) // String is name volume or ""
};
```
Next pages each page will have to be the content of the chapters:
```rust
Page {
	index: index as i32, // index chapter
	text, // the html chapter
	base64, // String the name chapter
	url, // Url chapter
	..Default::default()
};
```

# Decode DRM support
Check this code
```rust

		let pages = chapters
		.map(|chapter_ref| {
			let chapter = chapter_ref.as_object().unwrap();

			let index = chapter.get("order").as_int().unwrap() as i32;
			let url = chapter.get("image_url").as_string().unwrap().read();
			let drm_data = chapter.get("drm_data").as_string().unwrap().read();

			let blocks = decode_drm(drm_data.replace('\n', "").trim()).unwrap();
			let base64 = {
				let mut s = String::from("[");
				for (i, rb) in blocks.iter().enumerate() {
					if i > 0 {
						s.push(',');
					}
					s.push_str(&format!(
						r#"{{"sx":0,"sy":-1,"dx":0,"dy":{},"width":0,"height":{}}}"#,
						rb.dy, rb.height
					));
				}
				s.push(']');
				s
			};

			Page {
				index,
				url,
				base64,
				..Default::default()
			}
		})
		.collect();

```

## Contributing
Contributions are welcome!

See [CONTRIBUTING.md](./.github/CONTRIBUTING.md) to get started with development.

## License
Licensed under either of Apache License, version 2.0 or MIT license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this repository by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
