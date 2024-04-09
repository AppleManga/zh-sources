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

const BASE_URL: &str = "https://www.kukanmanhua.com";

const FILTER_CATE: [&str; 20] = ["全部", "校园", "搞笑", "后宫", "生活", "恋爱", "霸总", "热血", "科幻", "古风", "真人", "悬疑", "穿越", "耽美", "恐怖", "修真", "百合", "伦理", "女主", "神幻"];
const FILTER_AREA: [&str; 6] = ["-1", "5", "4", "3", "2", "1"];
const FILTER_END: [&str; 3] = ["-1", "2", "1"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut cate = String::new();
	let mut area = String::new();
	let mut end = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"题材" => {
						cate = FILTER_CATE[index].to_string();
					}
					"地区" => {
						area = FILTER_AREA[index].to_string();
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
		format!("{}/booklist?page={}&cate={}&area={}&end={}",
			BASE_URL,
			page,
			encode_uri(cate),
			area,
			end
		)
	} else {
		format!("{}/search?keyword={}", BASE_URL, encode_uri(query.clone()))
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
			.read().replace("/book/", "");
		let cover = item.select(".mh-cover").attr("style").read().trim().replace("background-image: url(", "").replace(")", "") ;
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
	let url = format!("{}/book/{}", BASE_URL, id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let cover = html
		.select(".banner_detail_form .cover>img")
		.attr("src")
		.read();
	let title = html
		.select("h1")
		.text().read();
	let author = html
		.select(".banner_detail_form > div.info > p:nth-child(3)")
		.text()
		.read()
		.trim()
		.to_string()
		.replace("作者：", "")
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
		.select(".banner_detail_form > div.info > p:nth-child(5) > span > a")
		.array()
		.map(|a| a.as_node().unwrap().text().read().trim().to_string())
		.filter(|a| !a.is_empty())
		.collect::<Vec<String>>();
	let status = {
		let status_text = html
			.select(".banner_detail_form > div.info > p:nth-child(4) > span:nth-child(1) > span")
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
	let url = format!("{}/book/{}", BASE_URL, manga_id.clone());
	let html = Request::new(url.clone(), HttpMethod::Get).html()?;
	let mut chapters: Vec<Chapter> = Vec::new();

	for (index, item) in html.select("#detail-list-select>li").array().enumerate() {
		let item = match item.as_node() {
			Ok(item) => item,
			Err(_) => continue,
		};
		let id = item
			.select("a")
			.attr("href")
			.read()
			.split("/")
			.map(|a| a.to_string())
			.collect::<Vec<String>>()
			.pop()
			.unwrap();
		let title = item.select("a").text().read().trim().to_string();
		let chapter = (index + 1) as f32;
		let url = format!("{}/chapter/{}", BASE_URL, id);
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
	let url = format!(
		"{}/chapter/{}",
		BASE_URL,
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
