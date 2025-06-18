use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use cookie_store::Cookie;
use rsa::Oaep;
use rsa::pkcs8::DecodePublicKey;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::bapi::*;
use crate::client::Client;

const REFRESH_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDLgd2OAkcGVtoE3ThUREbio0Eg
Uc/prcajMKXvkCKFCWhJYJcLkcM2DKKcSeFpD/j6Boy538YXnR6VhcuUJOhH2x71
nzPjfdTcqMz7djHum0qSZA0AyCBDABUqCrfNgCiJ00Ra7GmRj+YCK1NJEuewlb40
JNrRuoEUXpabUzGB8QIDAQAB
-----END PUBLIC KEY-----"#;

#[derive(Clone)]
pub struct Session {
    pub client: Arc<Client>,
    login_info: Arc<Mutex<Option<LoginInfo>>>,
}

impl Session {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let client = Arc::new(Client::new(None)?);
        let session = Self {
            client,
            login_info: Arc::new(Mutex::new(None)),
        };
        // session.bypass_risk_check().await?;
        Ok(session)
    }

    async fn _bypass_risk_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        let response: ResponseData<Value> = self
            .client
            .get("https://api.bilibili.com/x/frontend/finger/spi")
            .send()
            .await?
            .json()
            .await?;
        if response.code != 0 {
            return Err(format!(
                "Failed to bypass risk check: GET finger/spi : {}",
                response.message
            )
            .into());
        }

        tracing::trace!("Bypassed risk check: {:?}", response);

        let data = response.data.ok_or("No data found in response")?;
        let buvid3 = data
            .get("b_3")
            .ok_or("No buvid3 field found in response")?
            .as_str()
            .ok_or("buvid3 is not a string")?;
        let url = &"https://bilibili.com".parse()?;
        self.client
            .add_cookie_str(&format!("buvid3={}", buvid3), &url);
        let buvid4 = data
            .get("b_4")
            .ok_or("No buvid4 field found in response")?
            .as_str()
            .ok_or("buvid4 is not a string")?;
        self.client
            .add_cookie_str(&format!("buvid4={}", buvid4), &url);
        Ok(())
    }

    pub async fn get_qrcode(&self) -> Result<QrCodeInfo, Box<dyn std::error::Error + Sync + Send>> {
        let response: ResponseData<QrCodeInfo> = self
            .client
            .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
            // .form(&form)
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("QR code response: {:?}", response);
        if response.code != 0 {
            return Err(format!("Failed to get QR code: {}", response.message).into());
        }
        Ok(response.data.ok_or("No QR code data found")?)
    }

    pub async fn check_login(
        &self,
        qrcode_key: &str,
    ) -> Result<Option<LoginInfo>, Box<dyn std::error::Error + Sync + Send>> {
        if let Some(info) = self.login_info.lock().await.deref() {
            return Ok(Some(info.clone()));
        }
        let params = BTreeMap::from([("qrcode_key", qrcode_key.to_string())]);
        let res: ResponseData<ResponseValue> = self
            .client
            .get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll")
            .query(&params)
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("Check login response: {:?}", res);

        match res {
            ResponseData {
                code: 0,
                data: Some(ResponseValue::Login(info)),
                ..
            } => match &info.code {
                0 => {
                    tracing::debug!("Login successful: {:?}", info);
                    self.login_info.lock().await.replace(info.clone());
                    Ok(Some(info))
                }
                86038 => {
                    tracing::trace!("QR code expired, waiting for new scan...");
                    Err("QR code expired, please scan again".to_string().into())
                }
                86090 => {
                    tracing::trace!("QR code scanned, waiting for confirmation...");
                    Ok(None)
                }
                86101 => {
                    tracing::trace!("QR code not scanned yet, waiting...");
                    Ok(None)
                }
                _ => Err(format!("Unexpected login code: {}, {}", info.code, info.message).into()),
            },

            data => {
                let res: ResponseValue = data.data.unwrap_or(ResponseValue::Value(Value::Null));
                tracing::error!("{}", serde_json::to_string_pretty(&res)?);
                Err(format!("Login failed: {:#?}", res).into())
            }
        }
    }

    pub async fn wait_for_login(
        &self,
        qrcode_key: &str,
        timeout: Duration,
    ) -> Result<LoginInfo, Box<dyn std::error::Error + Sync + Send>> {
        let start = SystemTime::now();
        loop {
            match self.check_login(qrcode_key).await {
                Ok(Some(info)) => {
                    return Ok(info);
                }
                Err(e) => {
                    return Err(e);
                }
                _ => {}
            }
            if SystemTime::now().duration_since(start)? > timeout {
                return Err("Login timed out".into());
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
            continue;
        }
    }

    pub async fn get_account(&self) -> Result<Account, Box<dyn std::error::Error + Sync + Send>> {
        let resp: ResponseData<Account> = self
            .client
            .get("https://api.bilibili.com/x/member/web/account")
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("Account response: {:?}", resp);
        if resp.code != 0 {
            return Err(format!("Failed to get account info: {}", resp.message).into());
        }
        let account = resp.data.ok_or("No account data found")?;
        Ok(account)
    }

    pub fn credentials(&self) -> Result<Credentials, Box<dyn std::error::Error + Sync + Send>> {
        let cookies = self.client.cookies();
        Credentials::from_cookies(&cookies)
    }

    pub async fn login_info(&self) -> Option<LoginInfo> {
        self.login_info.lock().await.clone()
    }

    fn get_correspond_path(ts: i64) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let pub_key = rsa::RsaPublicKey::from_public_key_pem(REFRESH_KEY)?;
        let msg = format!("refresh_{}", ts);
        let padding = Oaep::new::<sha2::Sha256>();
        let enc_data = pub_key.encrypt(&mut rand::rng(), padding, msg.as_bytes())?;
        Ok(hex::encode(enc_data))
    }

    async fn get_refresh_csrf(
        &self,
        correspond_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let url = format!("https://www.bilibili.com/correspond/1/{}", correspond_path);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get refresh csrf: {}", response.status()).into());
        }

        let body = response.text().await?;
        tracing::trace!("Response body: {}", body);
        // regex to extract the csrf token
        let re = regex::Regex::new(r#"<div id="1-name">([^<]+)</div>"#).unwrap();
        if let Some(captures) = re.captures(&body) {
            if let Some(csrf) = captures.get(1) {
                let csrf_value = csrf.as_str();
                tracing::trace!("Extracted csrf: {}", csrf_value);
                return Ok(csrf_value.to_string());
            }
        }
        Err("Failed to extract csrf token from response".into())
    }

    pub async fn refresh(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        tracing::trace!("Refresh info");
        let token = self
            .login_info()
            .await
            .ok_or("Login info not available")?
            .refresh_token
            .clone();
        let csrf = self.credentials()?.bili_jct;
        let info: ResponseData<RefreshInfo> = self
            .client
            .get("https://passport.bilibili.com/x/passport-login/web/cookie/info")
            .query(&BTreeMap::from([("csrf", csrf.clone())]))
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("Refresh info response: {:?}", info);
        if info.code != 0 {
            return Err(format!("Failed to refresh: {}", info.message).into());
        };
        let refresh_info = info.data.ok_or("No refresh data found")?;
        if !refresh_info.refresh {
            return Ok(());
        }
        let correspond_path = Self::get_correspond_path(refresh_info.timestamp)?;
        let refresh_csrf = self.get_refresh_csrf(&correspond_path).await?;

        let params = BTreeMap::from([
            ("csrf", csrf),
            ("refresh_csrf", refresh_csrf),
            ("source", "main_web".to_string()),
            ("refresh_token", token),
        ]);
        let response: ResponseData<Value> = self
            .client
            .post("https://passport.bilibili.com/x/passport-login/web/cookie/refresh")
            .form(&params)
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("Refresh cookie response: {:?}", response);
        if response.code != 0 {
            return Err(format!("Failed to refresh cookies: {}", response.message).into());
        }

        if let Some(data) = response.data {
            let mut login_info = self.login_info.lock().await;
            if let Some(info) = login_info.as_mut() {
                info.refresh_token = data
                    .get("refresh_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
            }
        } else {
            return Err("Login info not available".into());
        }
        Ok(())
    }
}

impl Serialize for Session {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("session", 2)?;

        let cookies = self.client.cookies();
        state.serialize_field("cookies", &cookies)?;

        let login_info = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { self.login_info.lock().await })
        });
        if let Some(info) = login_info.deref() {
            state.serialize_field("login_info", info)?;
        } else {
            state.serialize_field("login_info", &Value::Null)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for Session {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SessionData<'a> {
            cookies: Option<Vec<Cookie<'a>>>,
            login_info: Option<LoginInfo>,
        }

        let data = SessionData::deserialize(deserializer)?;

        let client =
            Arc::new(Client::new(None).map_err(|e| {
                serde::de::Error::custom(format!("Failed to create client: {}", e))
            })?);
        match data.cookies {
            None => {}
            Some(cookies) => {
                client.restore_cookies(cookies);
            }
        }

        let session = Session {
            client,
            login_info: Arc::new(Mutex::new(data.login_info)),
        };

        Ok(session)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub sessdata: String,
    pub bili_jct: String,
    pub dede_user_id: String,
    pub dede_user_id_ckmd5: String,
}

impl Credentials {
    /// Extracts credentials from cookies and initializes a Credentials struct.
    fn from_cookies(
        cookies: &Vec<Cookie>,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let sessdata = cookies
            .iter()
            .find(|cookie| cookie.name() == "SESSDATA")
            .map(|cookie| cookie.value())
            .ok_or("SESSDATA cookie not found")?
            .to_string();

        let bili_jct = cookies
            .iter()
            .find(|cookie| cookie.name() == "bili_jct")
            .map(|cookie| cookie.value())
            .ok_or("bili_jct cookie not found")?
            .to_string();

        let dede_user_id = cookies
            .iter()
            .find(|cookie| cookie.name() == "DedeUserID")
            .map(|cookie| cookie.value())
            .ok_or("DedeUserID cookie not found")?
            .to_string();

        let dede_user_id_ckmd5 = cookies
            .iter()
            .find(|cookie| cookie.name() == "DedeUserID__ckMd5")
            .map(|cookie| cookie.value())
            .ok_or("DedeUserID__ckMd5 cookie not found")?
            .to_string();

        Ok(Credentials {
            sessdata,
            bili_jct,
            dede_user_id,
            dede_user_id_ckmd5,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use qrcode::QrCode;
    use qrcode::render::svg;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_login() {
        let session = Session::new().await.unwrap();
        let qrcode_info = session
            .get_qrcode()
            .await
            .map_err(|e| format!("Failed to get QR code: {}", e))
            .unwrap();
        assert!(
            !qrcode_info.url.is_empty(),
            "QR code URL should not be empty"
        );
        tracing::debug!("QR Code: {:?}", qrcode_info);

        let qr = QrCode::new(qrcode_info.url).unwrap();
        let qr_string = qr
            .render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();
        tracing::debug!("QR code:\n{}", qr_string);
        let svg = qr
            .render()
            .min_dimensions(200, 200)
            .quiet_zone(true)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();

        let mut file = std::fs::File::create("test.qr.svg").unwrap();
        file.write_all(svg.as_bytes()).unwrap();
        let info = session
            .wait_for_login(&qrcode_info.qrcode_key, Duration::from_secs(60))
            .await
            .map_err(|e| format!("Failed to wait for login: {}", e))
            .unwrap();

        tracing::debug!("Login Info: {:?}", info);
        tracing::debug!("Cookies: {:?}", session.client.cookies());
        tracing::debug!("Credentials: {:?}", session.credentials().unwrap());
        let account = session.get_account().await.unwrap();
        tracing::debug!("Account Info: {:?}", account);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_session_serialization() {
        let session = Session::new().await.unwrap();
        session._bypass_risk_check().await.unwrap();
        let serialized = serde_json::to_string(&session).unwrap();
        tracing::info!("Serialized Session: {}", serialized);

        let cookies = session.client.cookies();

        let deserialized: Session = serde_json::from_str(&serialized).unwrap();

        deserialized.client.cookies().iter().for_each(|cookie| {
            // check if session cookies match
            assert!(
                cookies
                    .iter()
                    .any(|c| c.name() == cookie.name() && c.value() == cookie.value()),
                "Cookie mismatch: {}={}",
                cookie.name(),
                cookie.value()
            );
        })
    }

    #[test]
    fn test_get_correspond_path() {
        let ts = 1700000000; // Example timestamp
        let path = Session::get_correspond_path(ts).unwrap();
        tracing::debug!("Corresponding path for timestamp {}: {}", ts, path);
        assert!(!path.is_empty(), "Path should not be empty");
    }
}
