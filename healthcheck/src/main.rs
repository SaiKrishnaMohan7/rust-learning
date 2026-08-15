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
        let urls = vec![
            "https://www.rust-lang.org",
            "https://doc.rust-lang.org",
            "https://crates.io",
            "https://example.com",
            "https://httpbin.org/delay/2",
        ];
        let start = Instant::now();

        // synchronously fetching
        for url in urls {
            let report = check(url).await;
            println!("{:?}", report);
        }
        println!("TOTAL: {}ms", start.elapsed().as_millis());
    });
}
