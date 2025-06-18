use std::fmt::{Debug, Display};
use std::time::Duration;

use crate::bapi::*;
use crate::session::Session;

#[derive(Clone)]
pub struct User {
    pub account: Account,
    pub session: Session,
}

impl User {
    pub async fn new(session: Session) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let account = session.get_account().await?;
        Ok(Self { account, session })
    }

    pub async fn get_login_qrcode(
        &self,
    ) -> Result<QrCodeInfo, Box<dyn std::error::Error + Sync + Send>> {
        self.session
            .get_qrcode()
            .await
            .map_err(|e| format!("Failed to get login QR code: {}", e).into())
    }

    pub async fn wait_for_login(
        &mut self,
        qr_code_info: QrCodeInfo,
    ) -> Result<LoginInfo, Box<dyn std::error::Error + Sync + Send>> {
        let login_info = self
            .session
            .wait_for_login(&qr_code_info.qrcode_key, Duration::from_secs(180))
            .await?;
        self.update_user_info().await?;
        Ok(login_info)
    }

    pub async fn update_user_info(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.account = self.session.get_account().await?;
        Ok(())
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User {{ Account: {:?} }}", self.account)
    }
}

impl Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;

    use qrcode::QrCode;
    use qrcode::render::svg;
    use tokio::sync::Mutex;

    use super::*;
    use crate::wbi::WBI;

    #[tokio::test]
    async fn test_user_login() {
        let session = Session::new().await.expect("Failed to create session");
        let qr_code_info = session.get_qrcode().await.expect("Failed to get qr code");
        let qr = QrCode::new(qr_code_info.url.as_str()).expect("Failed to create qrcode");
        let svg = qr
            .render()
            .min_dimensions(200, 200)
            .quiet_zone(true)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();

        let mut file = std::fs::File::create("test.qr.svg").expect("Failed to create svg file");
        file.write_all(svg.as_bytes())
            .expect("Failed to write svg file");

        tracing::info!("QR code saved to test.qr.svg");
        let login_info = session
            .wait_for_login(&qr_code_info.qrcode_key, Duration::from_secs(180))
            .await
            .expect("Failed to wait for login");

        let mut user = User::new(session).await.expect("Failed to create user");
        user.update_user_info()
            .await
            .expect("Failed to update user info");
        tracing::info!("User login: {:?}", user);
    }
}
