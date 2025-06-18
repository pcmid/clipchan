use std::sync::Arc;
use std::time::Duration;

use cookie_store::Cookie;
use delegate::delegate;
use reqwest::Url;
use reqwest::header::HeaderMap;
use reqwest_cookie_store::CookieStoreMutex;

/// A client for making HTTP requests with cookie management.
#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    cookie_store: Arc<CookieStoreMutex>,
}

impl Client {
    /// Creates a new `Client` with optional default headers.
    /// # Arguments
    /// * `headers`: Optional headers to be set as default for the client.
    /// # Returns
    /// * `Result<Client, Box<dyn std::error::Error>>`: A result containing the client or an error.
    pub(crate) fn new(
        headers: Option<HeaderMap>,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let cookie_store = Arc::new(CookieStoreMutex::default());
        let client = reqwest::Client::builder()
            .default_headers(headers.unwrap_or_else(Self::default_header))
            .cookie_store(true)
            .cookie_provider(cookie_store.clone())
            .timeout(Duration::new(30, 0))
            .build()?;
        Ok(Self {
            client,
            cookie_store,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn add_cookie_str(&self, cookie_str: &str, url: &Url) {
        let mut store = self.cookie_store.lock().unwrap();
        store
            .parse(cookie_str, url)
            .map_err(|e| {
                tracing::error!("Failed to parse cookie string: {}", e);
            })
            .ok();
    }

    pub(crate) fn restore_cookies(&self, cookies: Vec<Cookie<'static>>) {
        let iter = cookies
            .iter()
            .map(|c| Ok::<Cookie, ()>(c.clone().into_owned()));
        let mut store = self.cookie_store.lock().unwrap();
        let new_cookie_store = cookie_store::CookieStore::from_cookies(iter, true).unwrap();
        store.clone_from(&new_cookie_store);
    }

    pub(crate) fn cookies(&self) -> Vec<Cookie<'static>> {
        let store = self.cookie_store.lock().unwrap();
        store
            .iter_any()
            .map(|cookie| cookie.clone().into_owned())
            .collect()
    }

    fn default_header() -> HeaderMap {
        let mut headers = HeaderMap::new();
        // Dont set default referer here
        // headers.insert("Referer", "https://www.bilibili.com/".parse().unwrap());
        headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
        headers.insert("Accept-Language", "zh-CN,zh;q=0.9".parse().unwrap());
        headers.insert("Pragma", "no-cache".parse().unwrap());
        headers.insert("Priority", "u=0, i".parse().unwrap());
        headers.insert("Sec-Ch-Ua", "Not;Brand=\"Chromium\";v=\"133\", Not;Brand=\"Microsoft Edge\";v=\"133\", Not;Brand=\"Google Chrome\";v=\"133\"".parse().unwrap());
        headers.insert("Sec-Ch-Ua-Mobile", "?0".parse().unwrap());
        headers.insert("Sec-Ch-Ua-Platform", "Windows".parse().unwrap());
        headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
        headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
        headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
        headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
        headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
        headers.insert("User-Agent",  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0".parse().unwrap());
        headers
    }
}

#[allow(dead_code)]
impl Client {
    delegate! {
        to self.client {
            pub(crate) fn get(&self, url: &str) -> reqwest::RequestBuilder;
            pub(crate) fn post(&self, url: &str) -> reqwest::RequestBuilder;
            pub(crate) fn put(&self, url: &str) -> reqwest::RequestBuilder;
            pub(crate) fn delete(&self, url: &str) -> reqwest::RequestBuilder;
            pub(crate) fn head(&self, url: &str) -> reqwest::RequestBuilder;
            pub(crate) fn patch(&self, url: &str) -> reqwest::RequestBuilder;
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderMap;

    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let headers = HeaderMap::new();
        let client = Client::new(Some(headers)).expect("Failed to create client");
        assert!(client.get("https://setcookie.net/").send().await.is_ok());
    }

    #[tokio::test]
    async fn test_set_cookie() {
        let client = Client::new(None).expect("Failed to create client");
        client.add_cookie_str(
            "test_cookie=value",
            &"https://echo.free.beeceptor.com".parse::<Url>().unwrap(),
        );
        let resp = client
            .get("https://echo.free.beeceptor.com")
            .send()
            .await
            .expect("Failed to get server response");
        assert!(
            resp.status().is_success(),
            "Failed to get response with set cookie"
        );
        let data = resp.json::<serde_json::Value>().await.unwrap();
        tracing::debug!("{:?}", data);
        assert_eq!(
            data.get("headers").unwrap().get("Cookie").unwrap(),
            "test_cookie=value"
        );
    }
}
