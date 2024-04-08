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

const WWW_URL: &str = "https://www.kanman.com";
const WAP_URL: &str = "https://m.kanman.com";
const WWW_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36";
const WAP_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Mobile/15E148 Safari/604.1";

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

	let list_url = if query.is_empty() {
		format!(
			"{}/api/getsortlist/?product_id=1&productname=kmh&platformname=wap&orderby=click&search_key=&comic_sort=&size=30&page={}",
				WAP_URL,
				page
		)
	} else {
		format!("{}/api/getsortlist/?product_id=1&productname=kmh&platformname=pc&search_key={}",
				WWW_URL,
				encode_uri(query.clone())
		)
	};

	let has_more = query.is_empty();

	let mut mangas: Vec<Manga> = Vec::new();

	if query.is_empty() {
		let json = Request::new(list_url, HttpMethod::Get).header("User-Agent", WAP_USER_AGENT).json()?.as_object()?;
		for item in json.get("data").as_object()?.get("data").as_array()? {
			let item_obj = item.as_object()?;
			let id = item_obj.get("comic_id").as_int().unwrap_or(0).to_string();
			let title = match item_obj.get("comic_name").as_string() {
				Ok(comic_name) => comic_name.read(),
				Err(_) => String::new(),
			};
			let cover = match item_obj.get("cover_img").as_string() {
				Ok(cover_img) => cover_img.read(),
				Err(_) => String::new(),
			};
			mangas.push(Manga {
				id,
				cover,
				title,
				..Default::default()
			});
		}
	} else {
		let json = Request::new(list_url, HttpMethod::Get).header("User-Agent", WWW_USER_AGENT).json()?.as_object()?;
		for item in json.get("data").as_array()? {
			let item_obj = item.as_object()?;
			let id = item_obj.get("comic_id").as_int().unwrap_or(0).to_string();
			let title = match item_obj.get("comic_name").as_string() {
				Ok(comic_name) => comic_name.read(),
				Err(_) => String::new(),
			};
			let cover = match item_obj.get("cover_img").as_string() {
				Ok(cover_img) => cover_img.read(),
				Err(_) => String::new(),
			};
			mangas.push(Manga {
				id,
				cover,
				title,
				..Default::default()
			});
		}
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})

}


#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/{}/", WAP_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", WAP_USER_AGENT).html()?;
	let cover = format!("https://image.yqmh.com/mh/{}.jpg", id.clone());
	let title = html
		.select("h1")
		.text().read().trim().to_string();
	let author = html
		.select("#js_comic_main > div:nth-child(2) > div.bd > div > div:nth-child(2) > ul > li")
		.text()
		.read()
		.trim()
		.split(" ")
		.map(|a| a.to_string())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = String::new();
	let description = html
		.select(".comic-describe")
		.text()
		.read()
		.trim()
		.to_string();
	let categories = html
		.select(".comic-tags > li > a")
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
	let url = format!("{}/{}/", WAP_URL, manga_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).header("User-Agent", WAP_USER_AGENT).html()?;
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select("#js_chapter_list>li").array().enumerate() {
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
		let url = format!("{}/{}/{}.html", WAP_URL, manga_id, id);
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
		"{}/api/getchapterinfov2?product_id=1&productname=kmh&platformname=wap&comic_id={}&chapter_newid={}&isWebp=0&quality=low",
		WAP_URL,
		manga_id.clone(),
		chapter_id.clone()
	);

	let json = Request::new(url.clone(), HttpMethod::Get).json()?.as_object()?;

	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in json.get("data").as_object()?.get("current_chapter").as_object()?.get("chapter_img_list").as_array()?.enumerate() {
		let url = item.as_string()?.to_string();
		let index = index as i32;
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}

	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", WAP_URL)
		.header("User-Agent", WAP_USER_AGENT);
}