#![windows_subsystem = "windows"]
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
mod database;
use dotenv::dotenv;

#[get("/v0/all")]
async fn all() -> impl Responder {
    println!("GET: All");
    let json = std::fs::read_to_string("music.json").unwrap();
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .keep_alive()
        .body(json)
}

#[get("/v0/cover/{id}")]
async fn cover(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    println!("GET: Cover ID: {}", id);
    let cover: Vec<u8> = match database::get_cover(id) {
        Ok(cover) => cover,
        Err(_) => return HttpResponse::NotFound().body("Cover not found"),
    };
    HttpResponse::Ok()
        .content_type("image/png")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .keep_alive()
        .body(cover)
}

#[get("/tracks/{id}")]
async fn track(path: web::Path<String>) -> impl Responder {
    let dir = std::env::var("MUSIC_DIR").unwrap();
    let id = path.into_inner();
    println!("GET: Track ID: {}", id);
    let song: String = match database::get_song(id) {
        Ok(song) => song,
        Err(_) => return HttpResponse::NotFound().body("Song not found"),
    };
    let file = std::fs::read(format!("{}\\{}", dir, song)).unwrap();
    HttpResponse::Ok()
        .content_type("audio/mpeg")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .keep_alive()
        .body(file)
}

#[get("/app")]
async fn app() -> impl Responder {
    println!("GET: App");
    let path =
        "C:\\Users\\Sergio\\flutter\\soncore\\build\\app\\outputs\\flutter-apk\\app-release.apk";
    HttpResponse::Ok()
        .content_type("application/vnd.android.package-archive")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .keep_alive()
        .body(std::fs::read(path).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args[1] == "--update" || args[1] == "--u" {
            database::update_db().unwrap();
        }
    }
    // let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    dotenv().ok();
    // set these env variables if you wanna use SSL
    // let cert = "C:\\Certbot\\live\\music.soncore.net\\fullchain.pem";
    // let key = "C:\\Certbot\\live\\music.soncore.net\\privkey.pem";
    // println!("CERT_CHAIN: {}", cert);
    // println!("KEY: {}", key);
    // builder.set_private_key_file(key, SslFiletype::PEM).unwrap();
    // builder.set_certificate_chain_file(cert).unwrap();
    // env variable for the ip address (will implement a config file later)
    let addr = "127.0.0.1";

    println!("Starting server at: {}", addr);

    HttpServer::new(|| {
        App::new()
            .service(all)
            .service(track)
            .service(cover)
            .service(app)
    })
    .bind(format!("{}:42069", addr))?
    // change this to just bind if you're not using SSL (also mind the port)
    // .bind_openssl(format!("{}:42069", addr), builder)?
    .run()
    .await
}
