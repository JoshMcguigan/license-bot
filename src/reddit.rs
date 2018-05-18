extern crate rraw;
extern crate dotenv;

const REDDIT_POST : &'static str = include_str!("reddit_post.md");

pub fn post_comment_for_missing_license_file(access_token: &str, reddit_user_agent: &str, id: &str){
    let reddit_read_only = dotenv::var("REDDIT_READ_ONLY").unwrap().eq(&String::from("true"));
    if reddit_read_only {
        return;
    }

    rraw::reply(access_token, reddit_user_agent, id, REDDIT_POST);
}

pub fn find_in_comments(search_text: &str, comments: Vec<rraw::listing::Comment>) -> bool {
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
