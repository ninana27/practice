use reqwest::header::USER_AGENT;
use reqwest::Client;
use reqwest::Error;
use std::time::Duration;
use async_trait::async_trait;
use select::{
    document::Document,
    predicate::{Attr, Class, Name, Predicate},
};

pub struct CveDetailsSpider {
    client: Client,
    user_agent: String,
}

#[derive(Debug, Clone)]
pub struct Cve {
    cve_id: String,
    time: String,
}

impl CveDetailsSpider {
    fn normalize_url(&self, url: &str) -> String {
        let url = url.trim();

        if url.starts_with("//www.cvedetails.com") {
            return format!("https:{}", url);
        } else if url.starts_with("/") {
            return format!("https://www.cvedetails.com{}", url);
        }

        return url.to_string();
    }
}


impl CveDetailsSpider {
    pub fn new() -> Self {
        let timeout = Duration::from_secs(6);
        let user_agent = "Mozilla/5.0 AppleWebKit/537.36 Chrome/134.0.0.0 Safari/537.36".to_string();
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("spiders/cvedetails: Building HTTP client");

        CveDetailsSpider { client, user_agent }
    }
}

#[async_trait]
impl super::Spider for CveDetailsSpider {
    type Item = Cve;

    fn name(&self) -> String {
        String::from("cvedetails")
    }

    fn start_urls(&self) -> Vec<String> {
        vec!["https://www.cvedetails.com/vulnerability-list/vulnerabilities.html".to_string()]
    }

    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        log::info!("visiting: {}", url);

        let http_res = self.client
            .get(url)
            .header(USER_AGENT, &self.user_agent)
            .send()
            .await?
            .text()
            .await?;

        println!("{http_res}");

        let mut items = Vec::new();

        let document = Document::from(http_res.as_str());
        let rows = document.find(Attr("id", "searchresults").descendant(Class("border-top")));
        for row in rows {
            let cve_id = row.find(Name("a")).next().map(|n| n.text()).unwrap_or_default();
            let time = row.find(|n: &select::node::Node<'_>| n.attr("data-tsvfield") == Some("publishDate"))
            .next()
            .map(|n| n.text())
            .unwrap_or_default();

            let cve = Cve {
                cve_id,
                time,
            };
    
            items.push(cve);
        }

        let next_pages_links = document
            .find(Attr("id", "pagingb").descendant(Name("a")))
            .filter_map(|n| n.attr("href"))
            .map(|url| self.normalize_url(url))
            .collect::<Vec<String>>();

        Ok((items, next_pages_links))

    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        println!("{} {}", item.cve_id, item.time);

        Ok(())
    }
}