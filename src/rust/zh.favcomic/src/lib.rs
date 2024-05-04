#![no_std]
extern crate alloc;

use aidoku::{error::Result, prelude::*, std::{String, Vec}, Chapter, Filter, FilterType, Manga, MangaPageResult, Page, MangaStatus, MangaContentRating, MangaViewer};
use alloc::string::ToString;
use aidoku::helpers::uri::encode_uri;

mod helper;

const FILTER_TYPES: [&str; 4] = ["boy", "girl", "r18", "picture"];
const FILTER_ORIGIN: [&str; 5] = ["0", "2", "3", "1", "4"];
const FILTER_FINISHED: [&str; 3] = ["0", "1", "2"];
const FILTER_FREE: [&str; 3] = ["0", "1", "2"];
const FILTER_SORT: [&str; 2] = ["1", "2"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut types = String::new();
	let mut origin = String::new();
	let mut finished = String::new();
	let mut free = String::new();
	let mut sort = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"类型" => {
						types = FILTER_TYPES[index].to_string();
					}
					"地区" => {
						origin = FILTER_ORIGIN[index].to_string();
					}
					"进度" => {
						finished = FILTER_FINISHED[index].to_string();
					}
					"付费" => {
						free = FILTER_FREE[index].to_string();
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

	let url = helper::gen_explore_url(types.clone(), encode_uri(query.clone()), origin, finished, free, sort, page.to_string());

	let html = helper::get_html(url)?;

	// 自动签到
	if query.is_empty() && types == "boy" && page == 1 {
		helper::check_in()
	}

	let has_more = query.is_empty();

	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".comic_li").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select(".cover_box > a")
			.attr("href")
			.read().replace("/comic/detail/", "");
		let cover = item.select(".cover_box > a > img").attr("src").read();
		let title = item.select(".brief_box > div.title > a").attr("title").read().trim().to_string();
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
	let url = helper::gen_detail_url(id.clone());
	let html = helper::get_html(url.clone())?;
	let cover = html.select(".comic_cover")
		.attr("src")
		.read();
	let title = html
		.select(".title > h1")
		.text().read();
	let author = html
		.select(".toggle_box > .author > a")
		.array()
		.map(|val| val.as_node().expect("Failed to get author").text().read())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = String::new();
	let description = html
		.select(".intro_box > h2")
		.text()
		.read()
		.trim()
		.to_string().replace("作品介绍：", "");
	let categories = html
		.select(".tag_box > a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = {
		let status_text = html
			.select(".state_box > span:nth-child(3)")
			.text()
			.read()
			.trim()
			.to_string();
		if status_text.contains("完结") {
			MangaStatus::Completed
		} else if status_text.contains("连载中") {
			MangaStatus::Ongoing
		} else {
			MangaStatus::Unknown
		}
	};

	let tag_url = html
		.select(".tag_box > a:nth-child(1)")
		.attr("href").read();
	let nsfw = if tag_url.clone().contains("/r18") {
		MangaContentRating::Nsfw
	} else if tag_url.clone().contains("/picture") {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	let toon_icon = html.select(".comic_cover_box > div > div.toon_box > img")
		.attr("src")
		.read();
	let viewer = if toon_icon.contains("vtoon_icon") {
		println!("Scroll");
		MangaViewer::Scroll
	} else {
		println!("Rtl");
		MangaViewer::Rtl
	};

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
	let url = helper::gen_detail_url(manga_id.clone());
	let html = helper::get_html(url.clone())?;
	let mut chapters: Vec<Chapter> = Vec::new();
	for (index, item) in html.select(".item_box").array().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item
			.select("a")
			.attr("href")
			.read().replace("/comic/chapter/", "");
		if id.is_empty() {
			continue
		}
		let title = item.select(".title").text().read().trim().to_string();
		let mut scanlator = item.select("span:nth-child(2)").text().read().trim().to_string().replace("￥", "").replace("&nbsp;", "");
		if scanlator == "会员专享" {
			scanlator = "登录免费".to_string();
		} else {
			scanlator = format!("￥ {}", scanlator );
		}
		let chapter = (index + 1) as f32;
		let url = helper::gen_chapter_url(id.clone());
		chapters.push(Chapter {
			id,
			title,
			chapter,
			scanlator,
			url,
			..Default::default()
		});
	}
	chapters.reverse();
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = helper::gen_chapter_url(chapter_id.clone());
	let html = helper::get_html(url.clone())?;
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html.select(".unlazyload").array().enumerate() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let index = index as i32;
		let mut url = item.attr("data-src").read().trim().to_string();
		if url.is_empty() {
			url = item.attr("src").read().trim().to_string();
		}
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}
