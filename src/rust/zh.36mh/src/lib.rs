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

const BASE_URL: &str = "https://www.36mh.org";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36";

const FILTER_LIST: [&str; 5] = ["0", "1", "2", "3", "4"];
const FILTER_ORDER: [&str; 2] = ["hits", "addtime"];

const FILTER_FINISH: [&str; 3] = ["0", "1", "2"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut list = String::new();
	let mut order = String::new();
	let mut finish = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"分类" => {
						list = FILTER_LIST[index].to_string();
					}
					"排序" => {
						order = FILTER_ORDER[index].to_string();
					}
					"进度" => {
						finish = FILTER_FINISH[index].to_string();
					}
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	let url = if query.is_empty() {
		format!(
			"{}/category/order/{}/list/{}/finish/{}/page/{}",
			BASE_URL,
			order,
			list,
			finish,
			page
		)
	} else {
		format!("{}/index.php/search?key={}", BASE_URL, encode_uri(query.clone()))
	};

	let html = Request::new(url, HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let has_more = query.is_empty();
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".searchResult > .list > .item").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select(".img > a")
			.attr("href")
			.read().replace("/manhua/", "");
		let cover = item.select(".img > a > img").attr("src").read();
		let title = item.select(".img > a > img").attr("alt").read();
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
	let url = format!("{}/manhua/{}", BASE_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let cover = html
		.select(".comicInfo > .ib.cover > .img > img")
		.attr("src")
		.read();
	let title = html
		.select(".comicInfo > .ib.cover > .img > img")
		.attr("title")
		.read();
	let author = html
		.select(".comicInfo > div.ib.info > p:nth-child(2) > span")
		.text()
		.read()
		.replace("作  者：", "");
	let artist = String::new();
	// aidoku::prelude::println!("artist: {}", artist);
	let description = html
		.select(".ib.info > p.content")
		.text()
		.read()
		.replace("介绍:", "");
	let categories = html
		.select(".ib.info > p.gray > span.ib.l > a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = {
		let status_text = html.select(".ib.info > p.gray > span:nth-child(3)")
			.text()
			.read()
			.replace("状 态：", "");
		if status_text.contains("完结") {
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
	let url = format!("{}/manhua/{}", BASE_URL, manga_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", USER_AGENT).html()?;
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select("#chapterlistload > .list > a").array().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item
			.select(".j-chapter-link")
			.attr("href")
			.read().replace("/manhua/", "").replace(".html", "");
		let title = item.select(".j-chapter-link").text().read().trim().to_string();
		// aidoku::prelude::println!("id: {}", id);
		let chapter = (index + 1) as f32;
		let url = format!("{}/{}.html", BASE_URL, id);
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
	let url = format!("{}/manhua/{}.html", BASE_URL, chapter_id.clone());
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