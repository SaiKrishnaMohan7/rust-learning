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

        // Every future comes from the same async fn, hence, same type! so we use Vec<_> and let the compiler fill the type
        // If futures were coming from different functions then we'd have to Vec<Pin<Box<dyn Future<Output = Report>>>>
        let futures: Vec<_> = urls.iter().map(|url| check(url)).collect();
        // Fetching concurrently
        let reports = trpl::join_all(futures).await;

        for report in reports {
            println!("{report:?}");
        }
        println!("TOTAL: {}ms", start.elapsed().as_millis());
    });
}
