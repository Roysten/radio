use std::cmp;
use std::fs::{self, OpenOptions};

use crate::mpv_simple::{MpvCtx, MpvFormat};

use serde::{Deserialize, Serialize};

/*#[derive(Serialize, Deserialize, Debug)]
struct MetadataUpdate<'a> {
    #[serde(rename = "icy-br")]
    bitrate: Option<&'a str>,
    #[serde(rename = "icy-pub")]
    public: Option<&'a str>,
    #[serde(rename = "icy-description")]
    description: Option<&'a str>,
    #[serde(rename = "icy-audio-info")]
    audio_info: Option<&'a str>,
    #[serde(rename = "icy-url")]
    url: Option<&'a str>,
    #[serde(rename = "icy-genre")]
    genre: Option<&'a str>,
    #[serde(rename = "icy-name")]
    name: Option<&'a str>,
    #[serde(rename = "icy-title")]
    title: Option<&'a str>,
}*/

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

    #[serde(skip, default)]
    pub last_id: usize,
}

#[derive(Deserialize, Serialize)]
pub struct Player {
    pub cfg: PlayerCfg,

    #[serde(skip, default)]
    pub cfg_path: String,

    #[serde(skip, default)]
    mpv_ctx: Option<MpvCtx>,
}

unsafe impl Send for Player {}
unsafe impl Sync for Player {}

impl Stream {
    pub fn new(id: usize, name: String, url: String) -> Self {
        Stream { id, name, url }
    }
}

impl Player {
    pub fn from_file(path: &std::path::Path, mut mpv_ctx: MpvCtx) -> Self {
        mpv_ctx
            .observe_property(0, "metadata", MpvFormat::String)
            .expect("Failed to observe metadata property");
        match fs::read_to_string(path) {
            Ok(txt) => {
                let mut player = serde_json::from_str::<Player>(&txt)
                    .expect("Failed to parse configuration file");
                player.cfg_path = path.to_str().unwrap().to_string();
                player.mpv_ctx = Some(mpv_ctx);
                player.cfg.last_id = player
                    .cfg
                    .streams
                    .iter()
                    .fold(0, |acc, stream| cmp::max(acc, stream.id));
                player
            }
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

    pub fn add(&mut self, name: String, url: String) -> &Stream {
        self.cfg.last_id += 1;
        self.cfg
            .streams
            .push(Stream::new(self.cfg.last_id, name, url));
        self.dump_cfg();
        self.cfg.streams.last().unwrap()
    }

    pub fn play(&mut self, id: usize) -> Result<&Stream, ()> {
        if let Some(stream) = self.cfg.streams.iter().find(|x| x.id == id) {
            let next_url = stream.url.to_string();
            self.cfg.current = id - 1;
            self.mpv_ctx
                .as_mut()
                .unwrap()
                .command(&["loadfile", &next_url])
                .expect("Error opening URL");
            self.dump_cfg();
            Ok(stream)
        } else {
            Err(())
        }
    }

    pub fn next(&mut self) {
        if !self.cfg.streams.is_empty() {
            self.cfg.current = (self.cfg.current + 1) % self.cfg.streams.len();
            let next_url = self.get_current().unwrap().url.to_string();
            self.mpv_ctx
                .as_mut()
                .unwrap()
                .command(&["loadfile", &next_url])
                .expect("Error opening URL");
            self.dump_cfg();
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
            self.mpv_ctx
                .as_mut()
                .unwrap()
                .command(&["loadfile", &prev_url])
                .expect("Error opening URL");
            self.dump_cfg();
        }
    }

    pub fn get_current(&self) -> Option<&Stream> {
        self.cfg.streams.get(self.cfg.current)
    }

    pub fn delete(&mut self, id: usize) -> Option<Stream> {
        match self.cfg.streams.iter().position(|stream| stream.id == id) {
            Some(pos) => {
                self.dump_cfg();
                Some(self.cfg.streams.remove(pos))
            },
            None => None,
        }
    }

    fn dump_cfg(&self) {
        let open_result = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.cfg_path);

        if let Ok(f) = open_result {
            let _ = serde_json::to_writer_pretty(f, &self);
        }
    }
}
