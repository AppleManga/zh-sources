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

const BASE_URL: &str = "https://www.qiruiyaoye.cn";

const FILTER_TAGS: [&str; 25] = ["", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24"];
const FILTER_END: [&str; 3] = ["", "0", "1"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut tags = String::new();
	let mut end = String::new();

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
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	let url = if query.is_empty() {
		if end.is_empty() {
			format!(
				"{}/category/{}/page/{}.html",
				BASE_URL,
				tags,
				page
			)
		} else {
			format!(
				"{}/category/{}/end/{}/page/{}.html",
				BASE_URL,
				tags,
				end,
				page
			)
		}
	} else {
		format!("{}/search/{}/", BASE_URL, encode_uri(query.clone()))
	};

	let html = Request::new(url, HttpMethod::Get).html()?;
	let has_more = query.is_empty();
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".mh-list.col7>li").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select(".mh-item>a")
			.attr("href")
			.read().replace("/book/", "").replace("/", "");
		let cover = format!("{}{}", BASE_URL, item.select(".mh-cover").attr("style").read().trim().replace("background-image: url(", "").replace(")", "")) ;
		let title = item.select(".mh-item > a").attr("title").read();
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
	let url = format!("{}/book/{}/", BASE_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let cover = format!("{}{}", BASE_URL, html
		.select(".banner_detail_form .cover>img")
		.attr("src")
		.read());
	let title = html
		.select("h1")
		.text().read();
	let author = html
		.select(".banner_detail_form > div.info > p.subtitle > a")
		.text()
		.read()
		.trim()
		.split(" ")
		.map(|a| a.to_string())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = String::new();
	let description = html
		.select(".content")
		.text()
		.read()
		.trim()
		.to_string();
	let categories = html
		.select(".banner_detail_form > div.info > p:nth-child(4) > span > a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = {
		let status_text = html
			.select(".banner_detail_form > div.info > p:nth-child(3) > span:nth-child(1) > span")
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
	let url = format!("{}/book/{}/", BASE_URL, manga_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select("#detail-list-select>li").array().rev().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item
			.select("h2>a")
			.attr("href")
			.read()
			.split("/")
			.map(|a| a.replace(".html", ""))
			.collect::<Vec<String>>()
			.pop()
			.unwrap();
		let title = item.select("h2>a").text().read().trim().to_string().replace(" ", "").replace("", "");
		let chapter = (index + 1) as f32;
		let url = format!("{}/chapter/{}/{}.html", BASE_URL, manga_id, id);
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
		"{}/chapter/{}/{}.html",
		BASE_URL,
		manga_id.clone(),
		chapter_id.clone()
	);
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html.select(".lazy").array().enumerate() {
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
