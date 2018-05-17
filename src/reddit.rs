extern crate rraw;
extern crate dotenv;

pub fn post_comment_for_missing_license_file(access_token: &str, reddit_user_agent: &str, id: &str){
    let reddit_read_only = dotenv::var("REDDIT_READ_ONLY").unwrap().eq(&String::from("true"));
    if reddit_read_only {
        return;
    }

    rraw::reply(access_token, reddit_user_agent, id, &("Thanks for sharing your open source project, but it looks like you haven't specified a license.\n\n".to_owned()+
        &"> When you make a creative work (which includes code), the work is under exclusive copyright by default. Unless you include a license that specifies otherwise, nobody else can use, copy, distribute, or modify your work without being at risk of take-downs, shake-downs, or litigation. Once the work has other contributors (each a copyright holder), “nobody” starts including you.\n\n".to_owned() +
        "[choosealicense.com](https://choosealicense.com/) is a great resource to learn about open source software licensing."));
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
