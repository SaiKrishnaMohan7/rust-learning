use std::time::{Duration, Instant};

use trpl::{self, Either};
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

async fn check_with_timeout(url: &str, delay: u64) -> Result<Report, String> {
    match trpl::race(check(url), trpl::sleep(Duration::from_secs(delay))).await {
        Either::Left(report) => Ok(report),
        Either::Right(_) => Err(format!("{url} timed out after {delay} seconds")),
    }
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

        let futures: Vec<_> = urls.iter().map(|url| check_with_timeout(url, 1)).collect();
        let reports = trpl::join_all(futures).await;

        for report in reports {
            println!("{report:?}");
        }
        println!("TOTAL: {}ms", start.elapsed().as_millis());
    });
}
