use crossbeam::thread::scope;
use crossbeam_channel::bounded;
use crossbeam_channel::select;
use crossbeam_channel::tick;
use humantime::format_duration;
use reqwest;
use sentinel::config;
use sentinel::ring::RingBuf;
use sentinel::messenger::Messenger;
use sentinel::result::SResult;
use slack_api;
use std::io;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

fn main() -> SResult<()> {
    let config = config::read()?;
    let token = config.slack_token;
    let client = slack_api::default_client()
        .map_err(|err| format!("could not get Slack API client: {}", err))?;
    let members = fetch_members(&client, &token)?;
    let user_id = find_user_id(&members, &config.user_name)?;
    let messenger = Messenger::new(client, token, user_id)?;
    let mut buf = RingBuf::new(10);
    let start = Instant::now();
    messenger.send("Started")?;
    process_stdin(&start, &messenger, &mut buf)?;
    let mut s = String::new();
    s.push_str("Exited");
    push_output(&mut s, &buf.contents());
    messenger.send(&s).unwrap_or_else(|err| {
        eprintln!("error sending message to Slack: {}", err);
    });
    buf.clear();
    Ok(())
}

fn process_stdin(start: &Instant, messenger: &Messenger, buf: &mut RingBuf) -> SResult<()> {
    let (pipe_r, mut pipe_w) = pipe::pipe();
    let reader = BufReader::new(pipe_r);
    scope(|s| {
        let (line_s, line_r) = bounded(0);
        s.spawn(move |_| {
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        // Receiver should never disconnect before we've sent
                        // all lines.
                        line_s.send(line).unwrap()
                    }
                    Err(e) => eprintln!("error reading lines from stdin: {}", e),
                }
            }
        });
        s.spawn(move |_| {
            let ticker = tick(Duration::from_secs(10));
            loop {
                select! {
                    recv(line_r) -> msg =>
                        match msg {
                            Ok(line) => buf.push(&line),
                            Err(_) => break,
                        },
                    recv(ticker) -> _ => {
                        let mut s = String::new();
                        let ts = Duration::from_secs(start.elapsed().as_secs());
                        s.push_str(&format_duration(ts).to_string());
                        s.push_str(" elapsed");
                        push_output(&mut s, &buf.contents());
                        messenger.send(&s).unwrap_or_else(|err| {
                            eprintln!("error sending message to Slack: {}", err);
                        });
                        buf.clear();
                    }
                }
            }
        });
        tee(&mut io::stdin(), &mut io::stdout(), &mut pipe_w).unwrap_or_else(|err| {
            eprintln!("error teeing input: {}", err);
        });
        drop(pipe_w);
    })
    .map_err(|_err| format!("TODO"))?;
    Ok(())
}

fn push_output(s: &mut String, output: &str) {
    if output.is_empty() {
        s.push_str("; no new output")
    } else {
        s.push_str("\n```\n");
        s.push_str(output);
        s.push_str("```\n");
    }
}

fn tee(input: &mut dyn Read, output_0: &mut dyn Write, output_1: &mut dyn Write) -> SResult<()> {
    let mut buffer = [0; 1 << 20];
    loop {
        let n = input
            .read(&mut buffer)
            .map_err(|err| format!("stdin read error: {}", err))?;
        output_0
            .write(&buffer[..n])
            .map_err(|err| format!("stdout write error: {}", err))?;
        output_1
            .write(&buffer[..n])
            .map_err(|err| format!("pipe write error: {}", err))?;
        if n == 0 {
            break;
        }
    }
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
