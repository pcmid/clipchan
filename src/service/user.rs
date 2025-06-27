use bilive::bapi::Account;
use bilive::session::Session;

use crate::core::entity::user;
use crate::core::jwt;
use crate::data::UserData;

pub struct UserService {
    user_data: UserData,
}

impl UserService {
    pub fn new(user_data: UserData) -> Self {
        Self { user_data }
    }

    pub async fn get_login_qrcode(&self) -> anyhow::Result<bilive::bapi::QrCodeInfo> {
        let session = Session::new()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;
        let qrcode_info = session
            .get_qrcode()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get QR code: {}", e))?;
        Ok(qrcode_info)
    }

    pub async fn check_bilibili_login(&self, qrcode_key: &str) -> anyhow::Result<Option<Account>> {
        let session = Session::new()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;
        let login_info = session
            .check_login(qrcode_key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check login status: {}", e))?;
        if login_info.is_some() {
            let account = session
                .get_account()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get account info: {}", e))?;
            let user_model = self.save_user_info(&account, &session).await?;
            tracing::debug!("User logged in: {:?}", user_model);
            return Ok(Some(account));
        }
        Ok(None)
    }

    async fn save_user_info(
        &self,
        account: &Account,
        session: &Session,
    ) -> anyhow::Result<user::Model> {
        let session_data = serde_json::to_string(&session)
            .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;

        self.user_data
            .save_user_info(account.mid as i64, account.uname.clone(), session_data)
            .await
    }

    pub async fn clean_session(&self, user: &user::Model) -> anyhow::Result<()> {
        tracing::trace!("Cleaning session for user {:?}", user);
        self.user_data.clean_session(user).await
    }

    pub async fn _get_user_by_id(&self, id: i64) -> anyhow::Result<Option<user::Model>> {
        self.user_data._get_user_by_id(id).await
    }

    pub async fn get_user_by_mid(&self, mid: i64) -> anyhow::Result<Option<user::Model>> {
        self.user_data.get_user_by_mid(mid).await
    }

    pub async fn get_session(&self, user: &user::Model) -> anyhow::Result<Session> {
        let session: Session = serde_json::from_str(&user.session)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize session data: {}", e))?;
        session
            .refresh()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to refresh session: {}", e))?;
        Ok(session)
    }

    pub async fn get_session_and_refresh(&self, user: &user::Model) -> anyhow::Result<Session> {
        let session: Session = self.get_session(user).await?;
        match session.refresh().await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Failed to refresh session: {}", e);
                self.clean_session(user)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to clean session for user {}: {}", user.id, e);
                    })
                    .ok();
                return Err(anyhow::anyhow!("Session is invalid, please login again"));
            }
        }
        Ok(session)
    }

    pub async fn get_user_by_token(&self, token: &str) -> anyhow::Result<Option<user::Model>> {
        let claims =
            jwt::verify_token(token).map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;
        let user = self
            .get_user_by_mid(claims.mid)
            .await?
            .filter(|u| !u.session.is_empty());
        Ok(user)
    }

    pub async fn generate_token_for_user(&self, mid: i64) -> anyhow::Result<String> {
        let user = self
            .get_user_by_mid(mid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let token = jwt::create_token(user.mid, user.uname, 30)
            .map_err(|e| anyhow::anyhow!("Failed to create JWT token: {}", e))?;

        Ok(token)
    }

    // 管理员功能
    pub async fn list_all_users(&self) -> anyhow::Result<Vec<user::Model>> {
        self.user_data.list_all_users().await
    }

    pub async fn update_user_permissions(
        &self,
        user_id: i64,
        is_admin: Option<bool>,
        can_stream: Option<bool>,
        is_disabled: Option<bool>,
    ) -> anyhow::Result<user::Model> {
        self.user_data
            .update_user_permissions(user_id, is_admin, can_stream, is_disabled)
            .await
    }

    pub async fn check_user_permissions(&self, user: &user::Model) -> anyhow::Result<()> {
        if user.is_disabled {
            return Err(anyhow::anyhow!("User account is disabled"));
        }
        Ok(())
    }

    pub async fn check_admin_permissions(&self, user: &user::Model) -> anyhow::Result<()> {
        self.check_user_permissions(user).await?;
        if !user.is_admin {
            return Err(anyhow::anyhow!("Admin privileges required"));
        }
        Ok(())
    }

    pub async fn check_stream_permissions(&self, user: &user::Model) -> anyhow::Result<()> {
        self.check_user_permissions(user).await?;
        if !user.can_stream {
            return Err(anyhow::anyhow!("Streaming not allowed for this user"));
        }
        Ok(())
    }
}
