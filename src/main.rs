use reqwest;
use sentinel::config;
use slack_api;

type SResult<T> = Result<T, String>;

fn main() -> SResult<()> {
    let config = config::read()?;
    let token = config.slack_token;
    let client = slack_api::default_client()
        .map_err(|err| format!("could not get Slack API client: {}", err))?;
    let members = fetch_members(&client, &token)?;
    let user_id = find_user_id(&members, &config.user_name)?;
    send_message(&client, &token, &user_id, "Hello, world!")?;
    Ok(())
}

fn send_message(client: &reqwest::Client, token: &str, user_id: &str, text: &str) -> SResult<()> {
    let mut req = slack_api::chat::PostMessageRequest::default();
    req.channel = user_id;
    req.text = text;
    let _resp = slack_api::chat::post_message(client, token, &req)
        .map_err(|err| format!("could not post message: {}", err));
    Ok(())
}

fn find_user_id(members: &Vec<slack_api::User>, user_name: &str) -> SResult<String> {
    let user = members
        .iter()
        .find(|user| {
            user.name
                .as_ref()
                .map_or_else(|| false, |name| name == user_name)
        })
        .ok_or(format!("could not find user {}", user_name))?;
    let user_id = user
        .id
        .as_ref()
        .ok_or(format!("could not get ID of user {}", user_name))?;
    Ok(String::from(user_id))
}

fn fetch_members(client: &reqwest::Client, token: &str) -> SResult<Vec<slack_api::User>> {
    let req = slack_api::users::ListRequest { presence: None };
    let resp = slack_api::users::list(client, token, &req)
        .map_err(|err| format!("could not get user list from Slack: {}", err))?;
    resp.members
        .ok_or(String::from("could not get user list from Slack"))
}
