extern crate alloc;

use aidoku::{
    error::{AidokuError},
    prelude::*,
    std::{
        current_date,
        defaults::{defaults_get, defaults_set},
        net::{HttpMethod, Request},
        String,
    },
};
use alloc::string::ToString;
use aidoku::std::html::Node;
use aidoku::helpers::uri::encode_uri;

const WWW_URL: &str = "https://www.favcomic.com";
const API_URL: &str = "https://api.favcomic.com";

pub fn gen_request(url: String, method: HttpMethod) -> Request {
    let username = defaults_get("username").and_then(|v| v.as_string().map(|v| v.read())).unwrap_or_default();
    let password = defaults_get("password").and_then(|v| v.as_string().map(|v| v.read())).unwrap_or_default();
    let token = defaults_get("token").and_then(|v| v.as_string().map(|v| v.read())).unwrap_or_default();
    let login_time = defaults_get("login_time").unwrap().as_int().unwrap_or_default();
    let expires_i64: i64 = login_time + 604800;
    let now_time_i64: i64 = gen_time().parse().unwrap_or(0);
    // 有账号密码，但令牌或登录时间为空，或登录时间过期时重新登录
    let cookie = if !url.contains("login") &&
        !username.is_empty() && !password.is_empty() &&
        ( (token.is_empty() || login_time == -1) || (expires_i64 < now_time_i64) ) {
        login().unwrap()
    } else {
        token
    };
   Request::new(url, method)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .header("Content-Type", "application/x-www-form-urlencoded")
       .header("Cookie", format!("token={}", cookie).as_str())
}

pub fn login() -> Result<String, AidokuError> {
    let request = gen_request(gen_login_url(), HttpMethod::Post).header("Cookie", "");
    let username = defaults_get("username")?.as_string()?.read();
    let password = defaults_get("password")?.as_string()?.read();
    if username.is_empty() || password.is_empty() {
        return Ok("".to_string())
    }
    let body = format!("loginName={}&password={}", username, password);
    let request = request.body(body.as_bytes());
    let json = request.json()?;
    let data = json.as_object()?;
    // 判断是否登录失败
    let result = data.get("result").as_string()?.read();
    if result == "fail" {
        return Ok("".to_string())
    }
    let data = data.get("model").as_object()?;
    let token = data.get("token");
    let login_time = data.get("loginTime");
    defaults_set("token", token.clone());
    defaults_set("login_time", login_time.clone());
    Ok(token.as_string()?.read())
}

pub fn gen_login_url() -> String {
    format!("{}/{}", WWW_URL, "login")
}

pub fn gen_time() -> String {
    format!("{}000", (current_date() as i64).to_string())
}

pub fn get_html(url: String) -> Result<Node, AidokuError> {
    let request = gen_request(url, HttpMethod::Get);
    request.html()

}

pub fn gen_explore_url(cate_id: String, origin: String, finished: String, free: String, sort: String, page: i32) -> String {
    format!("{}/{}?keyword=&origin={}&finished={}&free={}&tag=0&sort={}&page={}",
            WWW_URL,
            cate_id,
            origin,
            finished,
            free,
            sort,
            page.to_string()
    )
}

pub fn gen_search_url(keyword: String, page: i32) -> String {
    format!("{}/search?keyword={}&page={}",
            WWW_URL,
            encode_uri(keyword),
            page.to_string()
    )
}

pub fn gen_detail_url(id: String) -> String {
    format!("{}/comic/detail/{}",
            WWW_URL,
            id,
    )
}

pub fn gen_chapter_url(id: String) -> String {
    format!("{}/comic/chapter/{}",
            WWW_URL,
            id,
    )
}

pub fn gen_checkin_url() -> String {
    format!("{}/console/app/user/signin",
            API_URL
    )
}

pub fn check_in() {
    let not_auto_check_in = !defaults_get("auto_check_in")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if not_auto_check_in {
        return;
    }
    let token = defaults_get("token").and_then(|v| v.as_string().map(|v| v.read())).unwrap_or_default();
    let request = Request::new(gen_checkin_url(), HttpMethod::Post)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .header("Token", token.as_str());
    let body = "timeZone=Asia/Shanghai";
    request.body(body).send();
}