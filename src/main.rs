use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
mod database;

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
    let id = path.into_inner();
    println!("GET: Track ID: {}", id);
    let song: String = match database::get_song(id) {
        Ok(song) => song,
        Err(_) => return HttpResponse::NotFound().body("Song not found"),
    };
    let file = std::fs::read(format!("D:\\Users\\Sergio\\Music\\Actual Music\\{}", song)).unwrap();
    HttpResponse::Ok()
        .content_type("audio/mpeg")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .keep_alive()
        .body(file)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // database::update_db().unwrap();
    builder
        .set_private_key_file(
            r"C:\Certbot\live\kwak.sytes.net\privkey.pem",
            SslFiletype::PEM,
        )
        .unwrap();
    builder
        .set_certificate_chain_file(r"C:\Certbot\live\kwak.sytes.net\fullchain.pem")
        .unwrap();

    HttpServer::new(|| App::new().service(all).service(track).service(cover))
        .bind_openssl("192.168.1.72:443", builder)?
        .run()
        .await
}
