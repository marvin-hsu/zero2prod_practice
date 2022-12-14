use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};

pub fn run() -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| 
        App::new()
        .route("/health_check", web::get().to(health_check)))
        .bind("0.0.0.0:8000")?
        .run();

    Ok(server)
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
