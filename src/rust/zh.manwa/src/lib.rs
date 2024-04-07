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

const FILTER_AREA: [&str; 7] = ["", "2", "3", "4", "5", "6", "1"];

const FILTER_END: [&str; 3] = ["", "2", "1"];

const FILTER_SORT: [&str; 4] = ["-1", "0", "1", "2"];

fn get_url() -> String {
	defaults_get("url").unwrap().as_string().unwrap().read()
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut area = String::new();
	let mut end = String::new();
	let mut sort = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"地区" => {
						area = FILTER_AREA[index].to_string();
					}
					"进度" => {
						end = FILTER_END[index].to_string();
					}
					"排序" => {
						sort = FILTER_SORT[index].to_string();
					}
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	let mut mangas: Vec<Manga> = Vec::new();

	if query.is_empty() {
		let url = format!(
			"{}/booklist?tag=&end={}&gender=-1&has_full=-1&area={}&sort={}&level=-1&page={}",
			get_url(),
			end,
			area,
			sort,
			page
		);
		let html = Request::new(url, HttpMethod::Get).html()?;
		for item in html.select(".manga-list-2>li").array() {
			let item = match item.as_node() {
				Ok(node) => node,
				Err(_) => continue,
			};
			let id = item
				.select(".manga-list-2-cover>a")
				.attr("href")
				.read().replace("/book/", "");
			let cover = item.select(".manga-list-2-cover-img").attr("src").read();
			let title = item.select(".manga-list-2-cover > a").attr("title").read();
			mangas.push(Manga {
				id,
				cover,
				title,
				..Default::default()
			});
		}
	} else {
		let url = format!("{}/search?keyword={}", get_url(), encode_uri(query.clone()));
		let html = Request::new(url, HttpMethod::Get).html()?;
		for item in html.select(".book-list>li").array() {
			let item = match item.as_node() {
				Ok(node) => node,
				Err(_) => continue,
			};
			let id = item
				.select(".book-list-cover>a")
				.attr("href")
				.read().replace("/book/", "");
			let cover = item.select(".book-list-cover-img").attr("data-original").read();
			let title = item.select(".book-list-cover > a").attr("title").read();
			mangas.push(Manga {
				id,
				cover,
				title,
				..Default::default()
			});
		}
	}

	let has_more = query.is_empty();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})

}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/book/{}", get_url(), id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let p_count = html.select(".detail-main-info>p").array().count();
	let author_select = if p_count < 7 { ".detail-main-info > p:nth-child(1) > span.detail-main-info-value > a".to_string() } else { ".detail-main-info > p:nth-child(2) > span.detail-main-info-value > a".to_string() };
	let status_select = if p_count < 7 { ".detail-main-info > p:nth-child(2) > span.detail-main-info-value".to_string() } else { ".detail-main-info > p:nth-child(3) > span.detail-main-info-value".to_string() };

	let cover = html
		.select(".detail-main-cover>img")
		.attr("data-original")
		.read();
	let title = html
		.select("h1")
		.text().read().trim().to_string();
	let author = html
		.select(author_select)
		.text()
		.read()
		.trim()
		.split(" ")
		.map(|a| a.to_string())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = String::new();
	let description = html
		.select(".detail-desc")
		.text()
		.read()
		.trim()
		.replace("免费成人H漫线上看", "");
	let categories = html
		.select(".detail-main-info-class > a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = match html
		.select(status_select)
		.text()
		.read()
		.as_str()
	{
		"连载中" => MangaStatus::Ongoing,
		"已完结" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
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
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/book/{}", get_url(), id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let list = html.select("#detail-list-select>li>a").array();
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in list.enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let chapter_id = item
			.select("a")
			.attr("href")
			.read().replace("/chapter/", "");
		let title = item.select("a").attr("title").read();
		let chapter = (index + 1) as f32;
		let url = format!(
			"{}/chapter/{}",
			get_url(),
			chapter_id.clone()
		);
		chapters.push(Chapter {
			id: chapter_id,
			title,
			chapter,
			url,
			..Default::default()
		});
	}
	chapters.reverse();

	Ok(chapters)
}

// 图片被加密了，需要解密后使用
// 算法在 res/ch.js 里
#[get_page_list]
fn get_page_list(_: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!(
		"{}/chapter/{}",
		get_url(),
		chapter_id.clone()
	);
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html.select(".content-img.lazy_img").array().enumerate() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let index = index as i32;
		let url = item.attr("data-r-src").read().trim().to_string();
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}
