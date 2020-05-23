use sentinel::config;

fn main() -> Result<(), String>{
    let config = config::read()?;
    println!("slack_token: {}", config.slack_token);
    Ok(())
}
