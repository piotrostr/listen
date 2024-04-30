pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1000000000.0
}

pub fn must_get_env(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => panic!("{} not found in environment", key),
    }
}
