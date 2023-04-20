use std::net::SocketAddr;

use actix_cors::Cors;
use actix_web::{web, HttpServer};

use self::stats_controller::*;

pub mod stats_controller;



fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::resource("/stats/{name}").route(web::get().to(stats_hello)))
            
    );
}


pub async fn start_api_server(addr: SocketAddr) -> anyhow::Result<()> {

    HttpServer::new(move || {
        
        actix_web::App::new()           
            .wrap(Cors::default().allow_any_origin())
            .configure(configure)
    })
    .bind(addr.to_string())?
    .run()
    .await?;

    Ok(())
}