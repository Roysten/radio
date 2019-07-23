mod http;
mod player;

use std::path::Path;
use std::sync::{Arc, Mutex};

use actix_web::{web, App, HttpServer};
use serde::{Deserialize, Serialize};

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

fn main() {
    let cfg_path = Path::new("my_cfg.json");

    let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Failed to init MPV builder");
    let mut mpv = mpv_builder.build().expect("Failed to build MPV handler");

    let player = Arc::new(Mutex::new(player::Player::from_file(cfg_path, mpv)));
    HttpServer::new(move || {
        App::new()
            .data(http::AppState{player: player.clone()})
            .route("/", web::get().to(http::index))
            .route("/playlist", web::get().to(http::get_playlist))
            .route("/stream", web::post().to(http::post_stream))
            .route("/stream", web::get().to(http::get_stream))
            .route("/stream/{id}", web::delete().to(http::delete_stream))
            .route("/next", web::put().to(http::put_next))
            .route("/prev", web::put().to(http::put_prev))
    })
    .workers(1)
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();

    /*mpv.command(&["loadfile", "http://stream.gal.io/arrow"]).expect("Error opening URL");
    //mpv.command(&["loadfile", "https://20043.live.streamtheworld.com/RADIO538.mp3?dist=radio-538_web"]);
    //mpv.command(&["loadfile", "https://icecast.omroep.nl/radio2-bb-mp3"]).expect("Error opening URL");

    'main: loop {
        while let Some(event) = mpv.wait_event(-1.0) {
            //println!("{:?}", event);
            match event {
                mpv::Event::Shutdown | mpv::Event::Idle => {
                    break 'main;
                },
                mpv::Event::PropertyChange{name, change: mpv::Format::Str(x), reply_userdata} => {
                    //println!("{}", x);
                    if let Ok(metadata) = serde_json::from_str::<MetadataUpdate>(x) {
                        if let Some(title) = metadata.title {
                            println!("Now playing: {}", title);
                        } else {
                            println!("Metadata update but no title was provided");
                        }
                    } else {
                        println!("Deserializing metadata failed");
                    }
                },
                _ => ()
            }
        }
    }*/
}
