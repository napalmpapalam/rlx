pub fn get_release_url(repo_url: String, version: String) -> String {
    let mut url_body = "/-/tags/";
    if repo_url.starts_with("https://github.com") {
        url_body = "/releases/tag/";
    }

    format!("{repo_url}{url_body}{version}")
}

pub fn get_compare_url(repo_url: String, previous: String, current: String) -> String {
    format!("{repo_url}/compare/{previous}...{current}")
}

pub fn substring(str: String, from: usize) -> String {
    str.chars()
        .skip(from)
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn is_empty_str(val: String) -> bool {
    val.is_empty() || val.trim().is_empty()
}

pub fn is_empty_str_vec(val: Vec<String>) -> bool {
    is_empty_str(val.join(""))
}
