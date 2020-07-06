use crossbeam_channel::Sender;
use crossbeam_channel::bounded;
use crate::result::SResult;
use std::process;
use std::thread;

pub struct Messenger {
    sender: Option<Sender<String>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Messenger {
    pub fn new(client: reqwest::Client, token: String, user_id: String) -> SResult<Messenger> {
        let (s, r) = bounded(0);
        let t = thread::spawn(move || {
            for text in r {
                let text = format!("`{}`: {}", process::id(), text);
                let mut req = slack_api::chat::PostMessageRequest::default();
                req.channel = &user_id;
                req.text = &text;
                let _resp = slack_api::chat::post_message(&client, &token, &req)
                    .map_err(|err| format!("could not post message: {}", err));
            }
        });
        let m = Messenger {
            sender: Some(s),
            thread: Some(t),
        };
        Ok(m)
    }

    pub fn send(&self, text: &str) -> SResult<()> {
        if let Some(s) = &self.sender {
            s.send(text.to_string())
                .map_err(|err| format!("could not send message: {}", err))?;
        }
        Ok(())
    }
}

impl Drop for Messenger {
    fn drop(&mut self) {
        let opt_s = std::mem::replace(&mut self.sender, None);
        if let Some(s) = opt_s {
            drop(s);
        }
        let opt_t = std::mem::replace(&mut self.thread, None);
        if let Some(t) = opt_t {
            // join() will return an error if the messaging thread panics. Just
            // propagate the panic to this thread.
            t.join().unwrap();
        }
    }
}
