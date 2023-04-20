use actix_web::{web, Responder, HttpResponse};


pub async fn stats_hello(
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    
    Ok(HttpResponse::Ok().body(format!("Hello {}", path.into_inner())))
}
