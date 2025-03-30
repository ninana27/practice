use std::{sync::Arc, time::Duration};


mod spiders;
pub use spiders::cvedetails::CveDetailsSpider;
mod crawler;
pub use crawler::Crawler;

#[tokio::main]
async fn main() {
    env_logger::init();

    let spider = Arc::new(spiders::cvedetails::CveDetailsSpider::new());
    let crawler = Crawler::new(Duration::from_millis(200), 2, 500);

    crawler.run(spider).await;
}
