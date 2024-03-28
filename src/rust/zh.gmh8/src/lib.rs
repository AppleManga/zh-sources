#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};
use alloc::string::ToString;

const BASE_URL: &str = "https://www.gmh8.com";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36";

const FILTER_TAGS: [&str; 43] = ["", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31", "48", "49", "50", "51", "52", "53", "294", "295", "296", "297", "298", "299", "300", "301", "302", "303"];
const FILTER_END: [&str; 3] = ["0", "1", "2"];
const FILTER_ORDER: [&str; 2] = ["hits", "addtime"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut tags = String::new();
	let mut end = String::new();
	let mut order = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"标签" => {
						tags = FILTER_TAGS[index].to_string();
					}
					"进度" => {
						end = FILTER_END[index].to_string();
					}
					"排序" => {
						order = FILTER_ORDER[index].to_string();
					}
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	let url = if query.is_empty() {
		if tags.is_empty() {
			format!(
				"{}/index.php/category/order/{}/finish/{}/page/{}",
				BASE_URL,
				order,
				end,
				page
			)
		} else {
			format!(
				"{}/index.php/category/order/{}/tags/{}/finish/{}/page/{}",
				BASE_URL,
				order,
				tags,
				end,
				page
			)
		}
	} else {
		format!("{}/index.php/search?key={}", BASE_URL, encode_uri(query.clone()))
	};

	let html = Request::new(url, HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let has_more = query.is_empty();
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".common-comic-item").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select(".cover")
			.attr("href")
			.read().replace("/index.php/comic/", "");
		let cover = item.select(".lazy").attr("data-original").read().trim().replace(" ", "");
		let title = item.select(".lazy").attr("alt").read();
		mangas.push(Manga {
			id,
			cover,
			title,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/index.php/comic/{}", BASE_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let cover = html
		.select(".de-info__cover>img")
		.attr("src")
		.read().trim().replace(" ", "");
	let title = html
		.select(".comic-title.j-comic-title")
		.text().read().trim().to_string();
	let author = html
		.select(".comic-author > .name > a")
		.text()
		.read()
		.trim()
		.split(" ")
		.map(|a| a.to_string())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = String::new();
	// aidoku::prelude::println!("artist: {}", artist);
	let description = html
		.select(".intro-total")
		.text()
		.read()
		.trim()
		.to_string();
	let categories = html
		.select(".comic-status > span:nth-child(1) > b > a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = MangaStatus::Unknown;
	let nsfw = MangaContentRating::Safe;
	let viewer = MangaViewer::Scroll;

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/index.php/comic/{}", BASE_URL, manga_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select(".j-chapter-item.chapter__item>a").array().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item
			.select(".j-chapter-link")
			.attr("href")
			.read().replace("/index.php/chapter/", "");
		let title = item.select(".j-chapter-link").text().read().trim().to_string();
		// aidoku::prelude::println!("id: {}", id);
		let chapter = (index + 1) as f32;
		let url = format!("{}/index.php/chapter/{}", BASE_URL, id);
		chapters.push(Chapter {
			id,
			title,
			chapter,
			url,
			..Default::default()
		});
	}
	chapters.reverse();
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/index.php/chapter/{}", BASE_URL, chapter_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html.select(".lazy-read").array().enumerate() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let index = index as i32;
		let url = item.attr("data-original").read().trim().replace(" ", "").to_string();
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}