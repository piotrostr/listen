pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1000000000.0
}

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1000000000.0) as u64
}
