mod http;
mod mpv_simple;
mod player;

use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex};

use actix_web::{web, App, HttpServer};
use getopts::Options;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.reqopt("o", "output", "Directory to store the user data", "DIR");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(msg) => {
            let exit_code =
                if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
                    print_usage(&program, opts);
                    0
                } else {
                    eprintln!("E: {}", msg.to_string());
                    1
                };
            std::process::exit(exit_code);
        }
    };

    let cfg_path_txt = matches.opt_str("o").unwrap();
    let cfg_path = Path::new(&cfg_path_txt);
    if !cfg_path.exists() {
        eprintln!("E: The provided path does not exist.");
        std::process::exit(1);
    }

    let mut mpv_ctx = mpv_simple::MpvCtx::create().expect("Failed to create MPV context");
    mpv_ctx.init().expect("Failed to initialize MPV context");

    let player = Arc::new(Mutex::new(player::Player::from_file(
        &cfg_path.join("radio.json"),
        mpv_ctx,
    )));
    HttpServer::new(move || {
        App::new()
            .data(http::AppState {
                player: player.clone(),
            })
            .route("/playlist", web::get().to(http::get_playlist))
            .route("/stream", web::post().to(http::post_stream))
            .route("/stream", web::get().to(http::get_stream))
            .route("/stream/{id}", web::delete().to(http::delete_stream))
            .route("/stream/{id}", web::put().to(http::put_play))
            .route("/next", web::put().to(http::put_next))
            .route("/prev", web::put().to(http::put_prev))
            .service(actix_files::Files::new("/", "web").index_file("index.html"))
    })
    .workers(1)
    .bind("0.0.0.0:8080")
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
