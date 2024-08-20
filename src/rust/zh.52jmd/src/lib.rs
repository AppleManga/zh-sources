#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::*,
	std::{
		defaults::defaults_get,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};
use alloc::string::ToString;

fn get_url() -> String {
	defaults_get("url").unwrap().as_string().unwrap().read()
}
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1";

const FILTER_CATALOG: [&str; 6] = ["all", "韩漫", "日漫", "3D漫画", "真人", "短篇"];
const FILTER_END: [&str; 3] = ["all", "serialized", "completed"];
const FILTER_ORDER: [&str; 2] = ["hits", "time"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut catalog = String::new();
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
					"分类" => {
						catalog = FILTER_CATALOG[index].to_string();
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
		format!(
			"{}/catalog/{}/ob/{}/st/{}/page/{}",
			get_url(),
			encode_uri(catalog),
			order,
			end,
			page
		)
	} else {
		format!("{}/cata.php?key={}", get_url(), encode_uri(query.clone()))
	};

	let html = Request::new(url, HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let has_more = query.is_empty();
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".module-item").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select(".module-item-pic>a")
			.attr("href")
			.read().replace("/manga/", "");
		let cover = item.select(".lazy.lazyload").attr("data-original").read();
		let title = item.select(".module-item-pic>a").attr("title").read();
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
	let url = format!("{}/manga/{}", get_url(), id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let cover = html
		.select(".mobile-play>.module-item-cover>.module-item-pic>img")
		.attr("data-original")
		.read();
	let title = html
		.select(".page-title")
		.text().read().trim().to_string();
	let author = html
		.select(".video-info-main > div:nth-child(2) > div > a")
		.text()
		.read()
		.trim()
		.split("&")
		.map(|a| a.to_string())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = html
		.select(".tag-link>a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>()
		.join(", ");
	// aidoku::prelude::println!("artist: {}", artist);
	let description = html
		.select(".video-info-content.vod_content>span")
		.text()
		.read()
		.trim()
		.to_string();
	let categories = html
		.select(".video-info > div.video-info-main > div:nth-child(1) > div > a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = {
		let status_text = html
			.select(".video-info > div.video-info-main > div:nth-child(3) > div")
			.text()
			.read()
			.trim()
			.to_string();
		if status_text.contains("已完结") {
			MangaStatus::Completed
		} else if status_text.contains("连载中") {
			MangaStatus::Ongoing
		} else {
			MangaStatus::Unknown
		}
	};

	let nsfw = MangaContentRating::Nsfw;
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
	let url = format!("{}/manga/{}", get_url(), manga_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select("#sort-item-3>a").array().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item
			.attr("href")
			.read()
			.split("/")
			.map(|a| a.to_string())
			.collect::<Vec<String>>()
			.pop()
			.unwrap();
		let title = item.select("span").text().read().trim().to_string();
		let chapter = (index + 1) as f32;
		let url = format!("{}/manga/{}/{}", get_url(), manga_id, id);
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
	let url = format!("{}/manga/{}/{}", get_url(), manga_id.clone(), chapter_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html.select("#main > div > center:nth-child(3) > div > img").array().enumerate() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let index = index as i32;
		let url = item.attr("data-original").read().trim().to_string();
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}