use std::{
    pin::Pin,
    time::{Duration, Instant},
};

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

async fn check_stub(name: &str) -> Report {
    trpl::sleep(Duration::from_millis(300)).await;
    Report {
        url: name.to_string(),
        bytes: 0,
        ms: 300,
    }
}

fn main() {
    trpl::block_on(async {
        let futures: Vec<Pin<Box<dyn Future<Output = Report>>>> = vec![
            Box::pin(check("https://example.com")),
            Box::pin(check_stub("fake-service")),
        ];
        let reports = trpl::join_all(futures).await;

        for report in reports {
            println!("{report:?}");
        }
    });
}
