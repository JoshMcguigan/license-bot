extern crate rraw;
extern crate time;
extern crate dotenv;
extern crate reqwest;
extern crate base64;

fn main() {
    let hours_to_go_back = 1;
    let max_reddit_submissions_to_review = 10;

    let reddit_user_agent = dotenv::var("REDDIT_USER_AGENT").unwrap();
    let reddit_username = dotenv::var("REDDIT_USERNAME").unwrap();
    let reddit_password = dotenv::var("REDDIT_PASSWORD").unwrap();
    let reddit_client_id = dotenv::var("REDDIT_CLIENT_ID").unwrap();
    let reddit_client_secret = dotenv::var("REDDIT_CLIENT_SECRET").unwrap();
    let reddit_read_only = dotenv::var("REDDIT_READ_ONLY").unwrap().eq(&String::from("true"));

    match rraw::authorize(&reddit_username, &reddit_password, &reddit_client_id, &reddit_client_secret, &reddit_user_agent) {
        Ok(auth_data) => {
            for subreddit in vec!["coolgithubprojects", "programming", "javascript"]{

                match rraw::new(&auth_data.access_token, &reddit_user_agent, subreddit, max_reddit_submissions_to_review) {
                    Ok(links) => {
                        for link in links.iter() {
                            let time_cutoff = time::now_utc().to_timespec().sec - 60 * 60 * hours_to_go_back;
                            if (link.created_utc as i64) < time_cutoff {
                                break;
                            }
                            println!("Reviewing post: {}", link.title);
                            println!("    - URL: {}", link.url);

                            let repo = get_repo_details_from_url(&link.url);

                            if let Some(repo) = repo {

                                println!("    - Found Github repository {}/{}", repo.username, repo.repo_name);

                                let license_exists = check_for_license(&repo);
                                let comments = rraw::comments(&auth_data.access_token, &reddit_user_agent, &link.subreddit, &link.id);
                                let license_discussion_found_in_comments = match comments {
                                    Ok(comments) => Ok(find_in_comments("license", comments)),
                                    Err(e) => Err(e)
                                };

                                match (license_exists, license_discussion_found_in_comments) {
                                    (Ok(false), Ok(false)) => {
                                        println!(" - Missing license found for post {} in subreddit {} with id {}", link.title, link.subreddit, link.id);
                                        post_comment_for_missing_license_file(&auth_data.access_token, &reddit_user_agent, &link.id, reddit_read_only);
                                    },
                                    (Err(e), Ok(_)) => {println!(" - {:?} while checking reddit post {}", e, link.title)},
                                    (Ok(_), Err(e)) => {println!(" - {:?} while checking reddit post {}", e, link.title)},
                                    (Err(e1), Err(e2)) => {println!(" - {:?} and {:?} while checking reddit post {}", e1, e2, link.title)},
                                    _ => {}
                                }
                            }
                        }
                    },
                    Err(e) => println!("error = {:?}", e)
                }
            }
        },
        Err(e) => println!("error = {:?}", e)
    };


}

fn post_comment_for_missing_license_file(access_token: &str, reddit_user_agent: &str, id: &str, reddit_read_only: bool){
    if reddit_read_only {
        return;
    }

    rraw::reply(access_token, reddit_user_agent, id, &("Thanks for sharing your open source project, but it looks like you haven't specified a license.\n\n".to_owned()+
        &"> When you make a creative work (which includes code), the work is under exclusive copyright by default. Unless you include a license that specifies otherwise, nobody else can use, copy, distribute, or modify your work without being at risk of take-downs, shake-downs, or litigation. Once the work has other contributors (each a copyright holder), “nobody” starts including you.\n\n".to_owned() +
        "[choosealicense.com](https://choosealicense.com/) is a great resource to learn about open source software licensing."));
}

#[derive(Debug)]
struct Repository {
    username: String,
    repo_name: String,
}

fn get_repo_details_from_url(url: &str) -> Option<Repository> {
    let github_project_url_prefix = "https://github.com";
    if url.starts_with(github_project_url_prefix) {
        let url_path = url.replace(github_project_url_prefix, "");
        let url_parts = url_path.split("/").collect::<Vec<&str>>();
        let username = url_parts.get(1);
        let repo_name = url_parts.get(2);
        match (username, repo_name) {
            (Some(username), Some(repo_name)) => Some(Repository {username: username.to_string(), repo_name: repo_name.to_string()}),
            _ => None
        }
    } else {
        None
    }
}

fn check_for_license(repo: &Repository) -> Result<bool, String> {
    match (check_for_license_from_github(repo), check_for_license_in_readme(repo)) {
        (Ok(a), Ok(b)) => Ok(a||b),
        (Err(e), Ok(_)) => Err(format!(" - {:?} while checking repo {:?}", e, repo)),
        (Ok(_), Err(e)) => Err(format!(" - {:?} while checking repo {:?}", e, repo)),
        (Err(e1), Err(e2)) => Err(format!(" - {:?} and {:?} while checking repo {:?}", e1, e2, repo))
    }
}

fn check_for_license_from_github(repo: &Repository) -> Result<bool, String> {
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

fn check_for_license_in_readme(repo: &Repository) -> Result<bool, String> {
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
                    Ok(text.contains("license"))
                },
                Err(e) => {Err(format!("Error {} while retrieving readme from Github", e))}
            }
        },
        Err(e) => {Err(format!("Error {} while retrieving readme from Github", e))}
    }
}

fn find_in_comments(search_text: &str, comments: Vec<rraw::listing::Comment>) -> bool {
    for comment in comments {
        if comment.body.to_lowercase().contains(&search_text.to_lowercase()) {
            println!(" - Found comment discussing license");
            return true;
        }
        if find_in_comments(search_text, comment.replies) {
            return true;
        }
    }

    false
}
