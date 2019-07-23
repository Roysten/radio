use std::sync::{Arc, Mutex};

use actix_web::{web, HttpResponse, Responder, Result};
use actix_web::http::uri::{Uri, Scheme};

use serde::{Deserialize, Serialize};

use crate::player::Player;

pub struct AppState {
    pub player: Arc<Mutex<Player>>,
}

#[derive(Deserialize, Serialize)]
pub struct StreamInfo {
    pub name: String,
    pub url: String,
}

pub fn index(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(data.player.lock().unwrap().cfg_path.clone())
}

pub fn get_playlist(data: web::Data<AppState>) -> Result<HttpResponse> {
    let guard = data.player.lock().unwrap();
    Ok(HttpResponse::Ok().json2(&guard.get_playlist()))
}

pub fn post_stream(info: web::Json<StreamInfo>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let new_stream = info.into_inner();

    match new_stream.url.parse::<Uri>() {
        Ok(ref uri) if uri.scheme_part() == Some(&Scheme::HTTP) || uri.scheme_part() == Some(&Scheme::HTTPS) => {
            let mut guard = data.player.lock().unwrap();
            Ok(HttpResponse::Ok().json2(guard.add(new_stream.name, new_stream.url)))
        },
        _ => {
            Ok(HttpResponse::BadRequest().body("URL invalid or unsupported"))
        },
    }
}

pub fn get_stream(data: web::Data<AppState>) -> impl Responder {
    let guard = data.player.lock().unwrap();
    HttpResponse::Ok().json2(&guard.get_current())
}

pub fn put_next(data: web::Data<AppState>) -> impl Responder {
    let mut guard = data.player.lock().unwrap();
    guard.next();
    HttpResponse::Ok().json2(&guard.get_current())
}

pub fn put_prev(data: web::Data<AppState>) -> impl Responder {
    let mut guard = data.player.lock().unwrap();
    guard.prev();
    HttpResponse::Ok().json2(&guard.get_current())
}

pub fn delete_stream(info: web::Path<usize>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let mut guard = data.player.lock().unwrap();
    Ok(HttpResponse::Ok().json2(&guard.delete(info.into_inner())))
}

