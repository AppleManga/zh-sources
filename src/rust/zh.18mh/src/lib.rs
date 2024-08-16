#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	helpers::{substring::Substring},
	prelude::*,
	std::{
		json,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};
use alloc::string::ToString;

const BASE_URL: &str = "https://18mh.org";

const FILTER_CATE: [&str; 6] = ["", "-genre/hanman", "-genre/zhenrenxiezhen", "-genre/riman", "-genre/aixiezhen", "-genre/hots"];


#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut cate = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"類型" => {
						cate = FILTER_CATE[index].to_string();
					}
					_ => continue,
				}
			}
			_ => continue,
		}
	}

	let url = if query.is_empty() {
		format!("{}/manga{}/page/{}",
				BASE_URL,
				cate,
				page
		)
	} else {
		format!("{}/s/{}?page={}", BASE_URL, encode_uri(query.clone()), page)
	};

	let html = Request::new(url, HttpMethod::Get).html()?;
	let has_more = query.is_empty();
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".cardlist > div").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item.select(".text-center > img").attr("alt").read();
		let cover = item.select(".text-center > img").attr("src").read();
		let title = item.select(".cardtitle").text().read();
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
	let url = format!("{}/manga/{}", BASE_URL, id.clone());
	let request = Request::new(url.clone(), HttpMethod::Get);
	let html_text = request.string()?;
	let html = aidoku::std::html::Node::new(&html_text)?;

	let info_json_text = html_text
		.substring_after("application/ld+json\">")
		.expect("Unable to get the substring")
		.substring_before("</script><script async")
		.expect("Unable to get the substring")
		.to_string();

	let json = json::parse(info_json_text)?;
	let data = json.as_object()?;

	let cover = data.get("image").as_string()?.read();
	let title = data.get("name").as_string()?.read();
	let author = html
		.select(".text-small.py-1.pb-2")
		.text()
		.read()
		.trim()
		.to_string()
		.replace("作者：", "");
	let artist = String::new();
	let description = data.get("description").as_string()?.read();
	let categories = html
		.select("div.block.text-left.mx-auto > div:nth-child(4) > a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = {
		let status_text = data.get("creativeWorkStatus").as_string()?.read();
		if status_text.contains("完結") {
			MangaStatus::Completed
		} else if status_text.contains("連載中") {
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
	let manga_url = format!("{}/manga/{}", BASE_URL, manga_id.clone());
	let manga_html = Request::new(manga_url.clone(), HttpMethod::Get).html()?;
	let mid = manga_html.select("#mangachapters").attr("data-mid").read();

	let chapterlist_url = format!("{}/manga/get?mid={}&mode=all", BASE_URL, mid.clone());
	let html = Request::new(chapterlist_url.clone(), HttpMethod::Get).html()?;

	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select("#allchapterlist>div").array().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item.select("a").attr("data-cs").read();
		let title = item.select("span.chaptertitle").text().read().trim().to_string();
		let chapter = (index + 1) as f32;
		let relative_url = item
			.select("a")
			.attr("href")
			.read();
		let url = format!("{}{}", BASE_URL, relative_url);
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
	let manga_url = format!("{}/manga/{}", BASE_URL, manga_id.clone());
	let manga_html = Request::new(manga_url.clone(), HttpMethod::Get).html()?;
	let mid = manga_html.select("#mangachapters").attr("data-mid").read();

	let url = format!(
		"{}/chapter/getcontent?m={}&c={}",
		BASE_URL,
		mid.clone(),
		chapter_id.clone()
	);
	let html = Request::new(url.clone(), HttpMethod::Get)
		.header("Referer", manga_url.as_str())
		.html()?;
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in html.select("#chapcontent > div > img").array().enumerate() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let index = index as i32;
		let url = {
			let src = item.attr("src").read().trim().to_string();
			if src.starts_with("http") {
				src
			} else {
				item.attr("data-src").read().trim().to_string()
			}
		};

		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}
