use std::cmp;
use std::fs::{self, OpenOptions};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::mpv_simple::{MpvCtx, MpvEvent, MpvFormat};

#[derive(Serialize, Deserialize, Debug)]
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
}

#[derive(Deserialize, Serialize)]
pub struct Stream {
    pub name: String,
    pub url: String,
    pub id: usize,
}

#[derive(Deserialize, Serialize, Default)]
pub struct PlayerCfg {
    pub streams: Vec<Stream>,
    pub current: usize,

    #[serde(skip, default)]
    pub last_id: usize,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Player {
    pub cfg: PlayerCfg,

    #[serde(skip, default)]
    cfg_path: String,

    #[serde(skip, default)]
    mpv_ctx: Option<Arc<Mutex<MpvCtx>>>,

    #[serde(skip, default)]
    now_playing: Arc<Mutex<String>>,

    #[serde(skip, default)]
    event_thread: Option<std::thread::JoinHandle<()>>,
}

unsafe impl Send for Player {}
unsafe impl Sync for Player {}
unsafe impl Send for MpvCtx {}

impl Stream {
    pub fn new(id: usize, name: String, url: String) -> Self {
        Stream { id, name, url }
    }
}

/**
 * Event thread
 */
fn read_events(rx: Receiver<()>, ctx: Arc<Mutex<MpvCtx>>, now_playing: Arc<Mutex<String>>) {
    loop {
        let _ = rx.recv();
        let mut guard = ctx.lock().unwrap();
        loop {
            match guard.wait_event(0.0) {
                Ok(MpvEvent::None) => break,
                Ok(MpvEvent::PropertyChange { change, .. }) => {
                    if let Ok(metadata) = serde_json::from_str::<MetadataUpdate>(&change) {
                        if let Some(title) = metadata.title {
                            let mut now_playing_guard = now_playing.lock().unwrap();
                            println!("{}", title);
                            *now_playing_guard = title.to_string();
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

impl Player {
    pub fn from_file(path: &std::path::Path, mut mpv_ctx: MpvCtx) -> Self {
        mpv_ctx
            .observe_property(0, "metadata", MpvFormat::String)
            .expect("Failed to observe metadata property");
        let mut player = match fs::read_to_string(path) {
            Ok(txt) => {
                let mut player = serde_json::from_str::<Player>(&txt)
                    .expect("Failed to parse configuration file");
                player.cfg_path = path.to_str().unwrap().to_string();
                player.mpv_ctx = Some(Arc::new(Mutex::new(mpv_ctx)));
                player.cfg.last_id = player
                    .cfg
                    .streams
                    .iter()
                    .fold(0, |acc, stream| cmp::max(acc, stream.id));
                player
            }
            Err(_) => Player {
                cfg_path: path.to_str().unwrap().to_string(),
                mpv_ctx: Some(Arc::new(Mutex::new(mpv_ctx))),
                now_playing: Arc::new(Mutex::new(String::new())),
                ..Default::default()
            },
        };

        let (tx, rx) = channel();

        let thread_ctx = player.mpv_ctx.clone();
        let thread_now_playing = player.now_playing.clone();

        player.event_thread = Some(std::thread::spawn(move || {
            read_events(rx, thread_ctx.unwrap(), thread_now_playing);
        }));

        let closure = move || {
            let _ = tx.send(());
        };

        {
            let mut guard = player.mpv_ctx.as_mut().unwrap().lock().unwrap();
            guard.set_wakeup_callback(closure);
        }

        player
    }

    fn play_stream(&mut self, stream: &str) {
        let mut guard = self.mpv_ctx.as_mut().unwrap().lock().unwrap();
        guard
            .command(&["loadfile", &stream])
            .expect("Error opening URL");
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
        let found = self.cfg.streams.iter().position(|x| x.id == id);
        if let Some(pos) = found {
            self.cfg.current = self.cfg.streams[pos].id - 1;
            let url = self.cfg.streams[pos].url.to_string();
            self.play_stream(&url);
            self.dump_cfg();
            Ok(&self.cfg.streams[pos])
        } else {
            Err(())
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
            }
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
