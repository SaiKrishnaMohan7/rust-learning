use std::time::Instant;

use trpl;
#[derive(Debug)]
pub struct Report {
    pub url: String,
    pub bytes: usize,
    pub ms: u128,
}

async fn check(url: &str) -> Report {
    let start = Instant::now();
    let response = trpl::get(url).await;
    let body = response.text().await;

    return Report {
        url: url.to_string(),
        bytes: body.len(),
        ms: start.elapsed().as_millis(),
    };
}

fn main() {
    trpl::block_on(async {
        let report = check("https://www.rust-lang.org").await;
        println!("{:?}", report);
    });
}
