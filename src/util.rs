use actix_web::{get, HttpResponse, Responder};

pub fn env(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| panic!("{} env var not set", var))
}

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1000000000.0
}

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1000000000.0) as u64
}

#[get("/healthz")]
pub async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("im ok")
}
