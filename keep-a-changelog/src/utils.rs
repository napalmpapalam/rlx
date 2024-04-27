use crate::token::{Token, TokenKind};
use eyre::{eyre, Result};

pub fn get_git_release_url(repo_url: String, version: String) -> String {
    let mut url_body = "/-/tags/";
    if repo_url.starts_with("https://github.com") {
        url_body = "/releases/tag/";
    }

    format!("{repo_url}{url_body}{version}")
}

pub fn get_git_compare_url(repo_url: String, previous: String, current: String) -> String {
    format!("{repo_url}/compare/{previous}...{current}")
}

pub fn substring(str: String, from: usize) -> String {
    let result: String = str.chars().skip(from).collect();
    result.trim().to_string()
}

pub fn is_empty_str(val: String) -> bool {
    val.is_empty() || val.trim().is_empty()
}

pub fn is_empty_str_vec(val: Vec<String>) -> bool {
    is_empty_str(val.join(""))
}

/// Return the next text content
pub fn get_text_content(tokens: &mut Vec<Token>) -> Result<Option<String>> {
    let mut lines: Vec<String> = vec![];
    let token_types = vec![TokenKind::P, TokenKind::Li];

    while tokens.first().is_some() {
        let first = tokens.first().unwrap();

        if !token_types.iter().any(|tt| *tt == first.kind) {
            break;
        }

        let token = tokens.remove(0);

        if token.kind == TokenKind::Li {
            lines.push(format!("- {}", token.content.join("\n")));
        } else {
            lines.push(token.content.join("\n"));
        }
    }

    let result = lines.join("\n");
    if result.is_empty() {
        return Ok(None);
    }

    Ok(Some(result))
}

/// Returns the content of a token
pub fn get_content(
    tokens: &mut Vec<Token>,
    kinds: Vec<TokenKind>,
    required: bool,
) -> Result<Option<String>> {
    let first = tokens.first();

    if first.is_none() || (first.is_some() && !kinds.iter().any(|k| *k == first.unwrap().kind)) {
        if required {
            return Err(eyre!("Required token missing in: {:?}", tokens[0].line));
        }
        return Ok(None);
    }

    let result = tokens.remove(0).content.join("\n");
    if result.is_empty() {
        if required {
            return Err(eyre!("Required token is empty in: {:?}", tokens[0].line));
        }
        return Ok(None);
    }

    Ok(Some(result))
}
