use dirs;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub slack_token: String,
}

pub fn read() -> Result<Config, String>{
    let mut path_buf = dirs::home_dir()
        .ok_or(String::from("could not determine home directory"))?;
    path_buf.push(".sentinel");
    path_buf.push("config");
    path_buf.set_extension("toml");
    let file = File::open(&path_buf)
        .map_err(|err| {
            format!("error opening {}: {}", path_buf.to_string_lossy(), err)
        })?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)
        .map_err(|err| {
            format!("error reading {}: {}", path_buf.to_string_lossy(), err)
        })?;
    let config: Config = toml::from_str(contents.as_str())
        .map_err(|err| {
            format!("error parsing {}: {}", path_buf.to_string_lossy(), err)
        })?;
    Ok(config)
}
