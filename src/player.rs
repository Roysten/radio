use std::cmp;
use std::fs;

use mpv::MpvHandler;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Stream {
    pub name: String,
    pub url: String,
    pub id: usize,
}

#[derive(Deserialize, Serialize)]
pub struct PlayerCfg {
    pub streams: Vec<Stream>,
    pub current: usize,

    #[serde(skip)]
    pub last_id: usize,
}

#[derive(Deserialize, Serialize)]
pub struct Player {
    pub cfg_path: String,
    pub cfg: PlayerCfg,

    #[serde(skip)]
    mpv_ctx: Option<MpvHandler>,
}

unsafe impl Send for Player { }
unsafe impl Sync for Player { }

impl Stream {

    pub fn new(id: usize, name: String, url: String) -> Self {
        Stream { id, name, url }
    }

}

impl Player {
    pub fn from_file(path: &std::path::Path, mut mpv_ctx: MpvHandler) -> Self {
        let _ = mpv_ctx.observe_property::<&str>("metadata", 0);
        match fs::read_to_string(path) {
            Ok(txt) => {
                let mut player = serde_json::from_str::<Player>(&txt).expect("Failed to parse configuration file");
                player.mpv_ctx = Some(mpv_ctx);
                player.cfg.last_id = player.cfg.streams.iter().fold(0, |acc, stream| cmp::max(acc, stream.id));
                player
            },
            Err(_) => Player {
                cfg_path: path.to_str().unwrap().to_string(),
                cfg: PlayerCfg {
                    streams: Vec::new(),
                    current: 0,
                    last_id: 0,
                },
                mpv_ctx: Some(mpv_ctx),
            },
        }
    }

    pub fn get_playlist(&self) -> &[Stream] {
        &self.cfg.streams
    }

    pub fn add(&mut self, name :String, url: String) -> &Stream {
        self.cfg.last_id += 1;
        self.cfg.streams.push(Stream::new(self.cfg.last_id, name, url));
        self.cfg.streams.last().unwrap()
    }

    pub fn next(&mut self) {
        if !self.cfg.streams.is_empty() {
            self.cfg.current = (self.cfg.current + 1) % self.cfg.streams.len();
            let next_url = self.get_current().unwrap().url.to_string();
            self.mpv_ctx.as_mut().unwrap().command(&["loadfile", &next_url]).expect("Error opening URL");
        }
    }

    pub fn prev(&mut self) {
        if !self.cfg.streams.is_empty() {
            if self.cfg.current == 0 {
                self.cfg.current = self.cfg.streams.len() - 1;
            } else {
                self.cfg.current = (self.cfg.current - 1) % self.cfg.streams.len();
            }
            let prev_url = self.get_current().unwrap().url.to_string();
            self.mpv_ctx.as_mut().unwrap().command(&["loadfile", &prev_url]).expect("Error opening URL");
        }
    }

    pub fn get_current(&self) -> Option<&Stream> {
        self.cfg.streams.get(self.cfg.current)
    }

    pub fn delete(&mut self, id: usize) -> Option<Stream> {
        match self.cfg.streams.iter().position(|stream| stream.id == id) {
            Some(pos) => {
                Some(self.cfg.streams.remove(pos))
            },
            None => None,
        }
    }
}

