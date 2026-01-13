use crate::models::*;
use actix_web::{web, HttpResponse, Responder};

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json("OK")
}

pub async fn get_daily_review(path: web::Path<String>) -> impl Responder {
    let _date = path.into_inner();
    let reviews: Vec<LimitUpReview> = vec![];
    HttpResponse::Ok().json(reviews)
}
