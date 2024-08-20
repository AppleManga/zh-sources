#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	helpers::{substring::Substring},
	prelude::*,
	std::{
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};
use alloc::string::ToString;

const BASE_URL: &str = "http://www.pipimanhua.com";

const FILTER_CATALOG: [&str; 19] = ["", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut catalog = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"分类" => {
						catalog = FILTER_CATALOG[index].to_string();
					}
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	if query.is_empty() {
		let url = format!(
			"{}/sort/{}/{}",
			BASE_URL,
			catalog,
			page
		);

		let html = Request::new(url, HttpMethod::Get).html()?;
		let has_more = query.is_empty();
		let mut mangas: Vec<Manga> = Vec::new();

		for item in html.select(".store_left > div > ul > li").array() {
			let item = match item.as_node() {
				Ok(node) => node,
				Err(_) => continue,
			};
			let id = item
				.select(".img_span > a")
				.attr("href")
				.read().replace("/manhua/", "").replace("/", "");
			let cover = item.select(".img_span > a > img").attr("data-original").read();
			let title = item.select(".w100 > a > h2").text().read();
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
	} else {
		let get_search_path_html = Request::new(BASE_URL, HttpMethod::Get).html()?;
		let search_path = get_search_path_html.to_string()
			.substring_after("action=\"")
			.expect("Unable to get the substring")
			.substring_before("\"  target=\"_blank\" onsubmit")
			.expect("Unable to get the substring")
			.to_string();
		let url = format!("{}{}?searchkey={}&searchtype=all", BASE_URL, search_path, encode_uri(query.clone()));
		let html = Request::new(url, HttpMethod::Get).html()?;
		let has_more = false;
		let mut mangas: Vec<Manga> = Vec::new();

		for item in html.select("ul.flex > li").array() {
			let item = match item.as_node() {
				Ok(node) => node,
				Err(_) => continue,
			};
			let id = item
				.select(".img_span > a")
				.attr("href")
				.read().replace("/manhua/", "").replace("/", "");
			let cover = item.select(".img_span > a > img").attr("data-original").read();
			let title = item.select("a > h3").text().read();
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
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/manhua/{}/", BASE_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let cover = html.select("meta[property=\"og:image\"]").attr("content").read();
	let title = html.select("meta[property=\"og:title\"]").attr("content").read();
	let author = html.select("meta[property=\"og:novel:author\"]").attr("content").read();
	let artist = String::new();
	let description = html.select("meta[property=\"og:description\"]").attr("content").read();
	let category_string = html.select("meta[property=\"og:novel:category\"]").attr("content").read();
	let categories = category_string.split('|')
		.map(|s| s.trim().to_string())
		.filter(|s| !s.is_empty())
		.collect();
	let status = {
		let status_text = html.select("meta[property=\"og:novel:status\"]").attr("content").read();
		if status_text.contains("全本") {
			MangaStatus::Completed
		} else if status_text.contains("连载") {
			MangaStatus::Ongoing
		} else {
			MangaStatus::Unknown
		}
	};

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
	let url = format!("{}/manhua/{}/", BASE_URL, manga_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;

	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select("#ul_all_chapters > li").array().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let replace_string = format!(
			"/read/{}/",
			manga_id.clone()
		);
		let id = item.select("a").attr("href").read().replace(&replace_string, "").replace(".html", "");
		let title = item.select("span.chaptertitle").text().read().trim().to_string();
		let chapter = (index + 1) as f32;
		let url = format!("{}/read/{}/{}.html", BASE_URL, manga_id.clone(), id);
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
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/read/{}/{}.html", BASE_URL, manga_id.clone(), chapter_id.clone());

	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html.select(".imgpic > img").array().enumerate() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let index = index as i32;
		let src = item.attr("src").read();
		let data_original = item.attr("data-original").read();
		let url = if data_original.is_empty() {
			src
		} else {
			data_original
		};
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}