#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	helpers::{substring::Substring, uri::encode_uri},
	prelude::*,
	std::{
		json,
		defaults::defaults_get,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};
use alloc::string::ToString;

fn get_url() -> String {
	defaults_get("url").unwrap().as_string().unwrap().read()
}

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
		format!("{}/albums-index-page-{}.html",
				get_url(),
				page
		)
	} else {
		format!("{}/search/index.php?q={}&m=&syn=yes&f=_all&s=create_time_DESC&p={}",
				get_url(),
				encode_uri(query.clone()),
				page
		)
	};
	let html = Request::new(url, HttpMethod::Get).html()?;
	let has_more = true;
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".gallary_item").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select(".pic_box>a")
			.attr("href")
			.read()
			.split("-")
			.map(|a| a.replace(".html", ""))
			.collect::<Vec<String>>()
			.pop()
			.unwrap();
		let cover = format!("https:{}", item.select(".pic_box>a>img").attr("src").read());
		let title = item.select(".pic_box>a")
			.attr("title")
			.read()
			.trim()
			.to_string()
			.replace("<em>", "")
			.replace("</em>", "");
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

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {

	let mut cate_id = String::new();

	match listing.name.as_str() {
		"同人誌(全部)" => {
			cate_id.push_str("5");
		}
		"同人誌-漢化" => {
			cate_id.push_str("1");
		}
		"同人誌-日語" => {
			cate_id.push_str("12");
		}
		"同人誌-English" => {
			cate_id.push_str("16");
		}
		"同人誌-CG畫集" => {
			cate_id.push_str("2");
		}
		"同人誌-3D漫畫" => {
			cate_id.push_str("22");
		}
		"同人誌-Cosplay" => {
			cate_id.push_str("3");
		}
		"單行本(全部)" => {
			cate_id.push_str("6");
		}
		"單行本-漢化" => {
			cate_id.push_str("9");
		}
		"單行本-日語" => {
			cate_id.push_str("13");
		}
		"單行本-English" => {
			cate_id.push_str("17");
		}
		"雜誌&短篇(全部)" => {
			cate_id.push_str("7");
		}
		"雜誌&短篇-漢化" => {
			cate_id.push_str("10");
		}
		"雜誌&短篇-日語" => {
			cate_id.push_str("14");
		}
		"雜誌&短篇-English" => {
			cate_id.push_str("18");
		}
		"韓漫(全部)" => {
			cate_id.push_str("19");
		}
		"韓漫-漢化" => {
			cate_id.push_str("20");
		}
		"韓漫-其他" => {
			cate_id.push_str("21");
		}
		_ => return get_manga_list(Vec::new(), page),
	}

	let url = format!("{}/albums-index-page-{}-cate-{}.html",
					  get_url(),
					  page,
					  cate_id);

	let html = Request::new(url, HttpMethod::Get).html()?;
	let has_more = true;
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".gallary_item").array() {
		let item = match item.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let id = item
			.select(".pic_box>a")
			.attr("href")
			.read()
			.split("-")
			.map(|a| a.replace(".html", ""))
			.collect::<Vec<String>>()
			.pop()
			.unwrap();
		let cover = format!("https:{}", item.select(".pic_box>a>img").attr("src").read());
		let title = item.select(".pic_box>a").attr("title").read().trim().to_string();
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
	let url = format!("{}/photos-index-aid-{}.html", get_url(), id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let cover = format!("https://{}", html
		.select(".asTBcell.uwthumb>img")
		.attr("src")
		.read()
		.replace("//", ""));
	let title = html
		.select("h2")
		.text().read().trim().to_string();
	let author = html
		.select(".asTBcell.uwconn > label:nth-child(1)")
		.text()
		.read()
		.trim()
		.to_string()
		.replace("分類：", "")
		.replace("&", "／")
		.split("／")
		.map(|a| a.trim().to_string())
		.collect::<Vec<String>>()
		.join(", ");
	let artist = String::new();
	let description = html
		.select(".asTBcell.uwconn>p")
		.text()
		.read()
		.trim()
		.to_string()
		.replace("簡介：", "");
	let categories = html
		.select(".tagshow")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = MangaStatus::Unknown;
	let nsfw = MangaContentRating::Nsfw;
	let viewer = MangaViewer::Vertical;

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
	let url = format!("{}/photos-index-aid-{}.html", get_url(), id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut chapters: Vec<Chapter> = Vec::new();

	let title = html
		.select(".asTBcell.uwconn > label:nth-child(2)")
		.text()
		.read()
		.trim()
		.to_string()
		.replace("頁數：", "");
	let chapter = 1 as f32;
	let url = format!("{}/photos-list-aid-{}.html", get_url(), id);
	chapters.push(Chapter {
		id,
		title,
		chapter,
		url,
		..Default::default()
	});
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(manga_id: String, _: String) -> Result<Vec<Page>> {
	let url = format!("{}/photos-item-aid-{}.html", get_url(), manga_id.clone());
	let text = Request::new(url.clone(), HttpMethod::Get).string()?;
	let mut list_text = text
		.substring_after("\"page_url\":")
		.unwrap()
		.substring_before(",]")
		.unwrap()
		.to_string();
	list_text = format!("{}]", list_text);
	let list = json::parse(list_text)?.as_array()?;
	let mut pages: Vec<Page> = Vec::new();
	for (index, item) in list.enumerate() {
		let index = index as i32;
		let url = item.as_string()?.read();
		pages.push(Page {
			index,
			url,
			..Default::default()
		})
	}
	Ok(pages)
}