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

use core::str;
use base64::{engine::general_purpose, Engine};

const BASE_URL: &str = "https://www.veryim.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			_ => continue,
		}
	}

	let url = if query.is_empty() {
		format!(
			"{}/list/{}.html",
			BASE_URL,
			page
		)
	} else {
		format!("{}/statics/search.aspx?key={}&page={}", BASE_URL, encode_uri(query.clone()), page)
	};

	let html = Request::new(url, HttpMethod::Get).html()?;
	let has_more = query.is_empty();
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".panel-body.panel-recommand > div").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select("h4 > a")
			.attr("href")
			.read().replace("/manhua/", "").replace("/", "");
		let cover = item.select(".media-left.media-heading > a > img").attr("src").read();
		let title = item.select("h4 > a").attr("title").read();
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
	let url = format!("{}/manhua/{}/", BASE_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let cover = html
		.select(".pannel-body.info > div.info1 > img")
		.attr("src")
		.read();
	let title = html
		.select(".info2 > h1")
		.text().read();
	let author = html
		.select(".info2 > h3 > a")
		.text()
		.read()
		.trim()
		.split(" ")
		.map(|a| a.to_string())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = String::new();
	let description = html
		.select(".info2 > div > p")
		.text()
		.read()
		.trim()
		.replace("漫画简介：", "");
	let categories = Vec::new();
	let status = {
		let status_text = html.select("meta[property=\"og:novel:status\"]").attr("content").read();
		if status_text.contains("完结") {
			MangaStatus::Completed
		} else if status_text.contains("连载中") {
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

	for (index, item) in html.select(".list-charts>li").array().rev().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item
			.select("a")
			.attr("href")
			.read()
			.split("/")
			.map(|a| a.replace(".html", ""))
			.collect::<Vec<String>>()
			.pop()
			.unwrap();
		let title = item.select("a").attr("title").read();
		let chapter = (index + 1) as f32;
		let url = format!("{}/manhua/{}/{}.html", BASE_URL, manga_id, id);
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
	let url = format!(
		"{}/manhua/{}/{}.html",
		BASE_URL,
		manga_id.clone(),
		chapter_id.clone()
	);
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut pages: Vec<Page> = Vec::new();

	let image_list_base64 = html.to_string()
		.substring_after("var qTcms_S_m_murl_e=\"")
		.expect("Unable to get the substring")
		.substring_before("\";var qTcms_S_m_murl_e2")
		.expect("Unable to get the substring")
		.to_string();

	let data = general_purpose::STANDARD.decode(image_list_base64).unwrap();
	let image_list_str = str::from_utf8(&data)?;
	let image_list: Vec<&str> = image_list_str.split("$qingtiandy$").collect();
	for (index, image) in image_list.iter().enumerate() {
		let index = index as i32;
		let url = image.to_string();
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}
