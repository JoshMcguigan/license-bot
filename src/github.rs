extern crate reqwest;

#[derive(Debug, PartialEq)]
pub struct Repository {
    pub username: String,
    pub repo_name: String,
}

pub fn get_repo_details_from_url(url: &str) -> Option<Repository> {
    let github_project_url_prefix = "https://github.com/";
    if url.starts_with(github_project_url_prefix) {
        let url_path = url.trim_left_matches(github_project_url_prefix).trim_right_matches("/");
        let url_parts = url_path.split("/").collect::<Vec<&str>>();

        if url_parts.len() != 2 {return None}

        let username = url_parts.get(0);
        let repo_name = url_parts.get(1);

        match (username, repo_name) {
            (Some(username), Some(repo_name)) => Some(Repository {username: username.to_string(), repo_name: repo_name.trim_right_matches(".git").to_string()}),
            _ => None
        }
    } else {
        None
    }
}

pub fn check_for_license(repo: &Repository) -> Result<bool, String> {
    match (check_for_license_from_github(repo), check_for_license_in_readme(repo)) {
        (Ok(a), Ok(b)) => Ok(a||b),
        (Err(e), Ok(_)) => Err(format!(" - {:?} while checking repo {:?}", e, repo)),
        (Ok(_), Err(e)) => Err(format!(" - {:?} while checking repo {:?}", e, repo)),
        (Err(e1), Err(e2)) => Err(format!(" - {:?} and {:?} while checking repo {:?}", e1, e2, repo))
    }
}

pub fn check_for_license_from_github(repo: &Repository) -> Result<bool, String> {
    let github_license_url = format!("https://api.github.com/repos/{}/{}/license", repo.username, repo.repo_name);
    let client = reqwest::Client::new();
    let res = client.get(&github_license_url)
        .header(reqwest::header::UserAgent::new("User-Agent: license-bot".to_owned()))
        .send();
    match res {
        Ok(res) => match res.status() {
            reqwest::StatusCode::Ok => Ok(true),
            reqwest::StatusCode::NotFound => Ok(false),
            other => Err(format!("Unexpected status code {} while retrieving license data from Github", other))
        },
        Err(e) => {Err(format!("Error {} while retrieving license data from Github", e))}
    }
}

pub fn check_for_license_in_readme(repo: &Repository) -> Result<bool, String> {
    let github_readme_url = format!("https://api.github.com/repos/{}/{}/readme", repo.username, repo.repo_name);
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::Headers::new();
    headers.set(reqwest::header::UserAgent::new("User-Agent: license-bot".to_owned()));
    headers.set_raw("Accept", "application/vnd.github.3.raw");
    let res = client.get(&github_readme_url)
        .headers(headers)
        .send();
    match res {
        Ok(mut res) => {
            match res.text() {
                Ok(text) => {
                    Ok(text.to_lowercase().contains(&"license".to_lowercase()))
                },
                Err(e) => {Err(format!("Error {} while retrieving readme from Github", e))}
            }
        },
        Err(e) => {Err(format!("Error {} while retrieving readme from Github", e))}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_repo_details_from_simple_github_url() {
        assert_eq!(
            get_repo_details_from_url("https://github.com/JoshMcguigan/license-bot"),
            Some(Repository { username: "JoshMcguigan".to_string(), repo_name: "license-bot".to_string() }));
    }

    #[test]
    fn get_none_repo_details_from_non_github_url() {
        assert_eq!(
            get_repo_details_from_url("https://google.com/JoshMcguigan/license-bot"),
            None);
    }

    #[test]
    fn get_repo_details_from_simple_github_url_stips_trailing_git() {
        assert_eq!(
            get_repo_details_from_url("https://github.com/JoshMcguigan/license-bot.git"),
            Some(Repository { username: "JoshMcguigan".to_string(), repo_name: "license-bot".to_string() }));
    }

    #[test]
    fn get_none_repo_details_for_deep_linked_github_url() {
        assert_eq!(
            get_repo_details_from_url("https://github.com/JoshMcguigan/license-bot/blob/master/README.md"),
            None);
    }

    #[test]
    fn get_repo_details_from_simple_github_url_with_trailing_slash() {
        assert_eq!(
            get_repo_details_from_url("https://github.com/JoshMcguigan/license-bot/"),
            Some(Repository { username: "JoshMcguigan".to_string(), repo_name: "license-bot".to_string() }));
    }
}
