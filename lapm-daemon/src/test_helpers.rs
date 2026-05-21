use rand::RngExt as _;

pub fn random_string(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

        let mut rng = rand::rng();
        (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
}

pub fn random_opt<T>(value: T) -> Option<T> {
    let mut rng = rand::rng();
    if rng.random_bool(1.0/2.0) {
        Some(value)
    } else {
        None
    }
}
