extern crate rraw;
extern crate time;
extern crate dotenv;
extern crate reqwest;
extern crate base64;

mod github;
mod reddit;

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

                            let repo = github::get_repo_details_from_url(&link.url);

                            if let Some(repo) = repo {

                                println!("    - Found Github repository {}/{}", repo.username, repo.repo_name);

                                let license_exists = github::check_for_license(&repo);
                                let comments = rraw::comments(&auth_data.access_token, &reddit_user_agent, &link.subreddit, &link.id);
                                let license_discussion_found_in_comments = match comments {
                                    Ok(comments) => Ok(reddit::find_in_comments("license", comments)),
                                    Err(e) => Err(e)
                                };

                                match (license_exists, license_discussion_found_in_comments) {
                                    (Ok(false), Ok(false)) => {
                                        println!(" - Missing license found for post {} in subreddit {} with id {}", link.title, link.subreddit, link.name);
                                        reddit::post_comment_for_missing_license_file(&auth_data.access_token, &reddit_user_agent, &link.name, reddit_read_only);
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
