use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
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
    let path = std::env::var("APP_PATH").unwrap();
    HttpResponse::Ok()
        .content_type("application/vnd.android.package-archive")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .keep_alive()
        .body(std::fs::read(path).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // database::update_db().unwrap();
    dotenv().ok();
    // set these env variables if you wanna use SSL
    let cert = std::env::var("CERT_CHAIN").unwrap();
    let key = std::env::var("KEY").unwrap();
    // env variable for the ip address (will implement a config file later)
    let addr = std::env::var("ADDR").unwrap();

    builder.set_private_key_file(key, SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file(cert).unwrap();

    HttpServer::new(|| {
        App::new()
            .service(all)
            .service(track)
            .service(cover)
            .service(app)
    })
    // change this to just bind if you're not using SSL (also mind the port)
    .bind_openssl(format!("{}:443", addr), builder)?
    .run()
    .await
}
