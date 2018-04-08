extern crate rawr;
extern crate time;
extern crate dotenv;
extern crate curl;
use rawr::prelude::*;
use rawr::structures::submission::Submission;
use curl::easy::{Easy, List};

fn main() {
    let hours_to_go_back = 2;
    let max_reddit_submissions_to_review = 60;

    let reddit_user_agent = dotenv::var("REDDIT_USER_AGENT").unwrap();
    let reddit_username = dotenv::var("REDDIT_USERNAME").unwrap();
    let reddit_password = dotenv::var("REDDIT_PASSWORD").unwrap();
    let reddit_client_id = dotenv::var("REDDIT_CLIENT_ID").unwrap();
    let reddit_client_secret = dotenv::var("REDDIT_CLIENT_SECRET").unwrap();

    let client = RedditClient::new(&reddit_user_agent, PasswordAuthenticator::new(&reddit_client_id, &reddit_client_secret, &reddit_username, &reddit_password));
    let subreddit = client.subreddit("coolgithubprojects");
    let new_listing = subreddit.new(ListingOptions::default()).expect("Could not fetch post listing!");
    for reddit_post in new_listing.take(max_reddit_submissions_to_review) {
        let time_cutoff = time::now_utc().to_timespec().sec - 60 * 60 * hours_to_go_back;
        if reddit_post.created_utc() < time_cutoff {
            break;
        }
        println!("{}", reddit_post.title());
        match reddit_post.link_url() {
            Some(url) => check_repo(reddit_post, url),
            None => println!(" - Found post with title {} that has no url", reddit_post.title())
        }
    }
}

fn check_repo(reddit_post: Submission, url: String){
    let license_exists = check_repo_for_missing_license(url);
    let license_discussion_found_in_comments = find_in_comments("license", reddit_post.clone());

    match (license_exists, license_discussion_found_in_comments) {
        (Ok(false), false) => post_comment_for_missing_license_file(reddit_post),
        (Err(e), _) => println!(" - {} while checking reddit post {}", e, reddit_post.title()),
        _ => {}
    }
}

fn post_comment_for_missing_license_file(reddit_post: Submission){
    println!(" - Missing license found for post {}", reddit_post.title());
    reddit_post.reply(&("Thanks for sharing your open source project, but it looks like you haven't specified a license.\n\n".to_owned()+
        &"> When you make a creative work (which includes code), the work is under exclusive copyright by default. Unless you include a license that specifies otherwise, nobody else can use, copy, distribute, or modify your work without being at risk of take-downs, shake-downs, or litigation. Once the work has other contributors (each a copyright holder), “nobody” starts including you.\n\n".to_owned() +
        "[choosealicense.com](https://choosealicense.com/) is a great resource to learn about open source software licensing.")).expect("Posting failed!");
}

fn check_repo_for_missing_license(url: String) -> Result<bool, String> {
    let github_project_url_prefix = "https://github.com";
    if url.starts_with(github_project_url_prefix) {
        let url_path = url.replace(github_project_url_prefix, "");
        let url_parts = url_path.split("/").collect::<Vec<&str>>();
        let username = url_parts.get(1);
        let repo = url_parts.get(2);
        match (username, repo) {
            (Some(username), Some(repo)) => return check_for_license(username, repo),
            _ => return Err(format!("URL {} which doesn't follow expected URL pattern", url))
        }
    } else {
        return Err(format!("URL {} which doesn't follow expected URL pattern", url));
    }
}

fn check_for_license(username: &str, repo: &str) -> Result<bool, String> {
    let github_license_url = format!("https://api.github.com/repos/{}/{}/license", username, repo);
    let mut easy = Easy::new();
    easy.url(&github_license_url).unwrap();
    let mut list = List::new();
    list.append("User-Agent: license-bot").unwrap();
    easy.http_headers(list).unwrap();
    match easy.perform() {
        Ok(_) => match easy.response_code() {
            Ok(status_code) => Ok(status_code>=200 && status_code<=299),
            Err(e) => {Err(format!("Error {} while retrieving license data from Github", e))}
        },
        Err(e) => {Err(format!("Error {} while retrieving license data from Github", e))}
    }
}

fn find_in_comments<'a, I>(search_text: &str, commentable: I) -> bool
    where I: Commentable<'a>
{
    for comment in commentable.replies().expect("Could not get replies") {
        let comment_body = comment.body().unwrap();
        if comment_body.to_lowercase().contains(&search_text.to_lowercase()) {
            return true;
        }
        if find_in_comments(search_text, comment) {
            return true;
        }
    }

    false
}
