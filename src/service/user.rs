use bilive::bapi::Account;
use bilive::session::Session;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::core::entity::user;
use crate::core::jwt;

pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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
        let existing_user = user::Entity::find()
            .filter(user::Column::Mid.eq(account.mid))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query database: {}", e))?;

        let session_data = serde_json::to_string(&session)
            .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;

        let now: sea_orm::prelude::DateTimeWithTimeZone = chrono::Utc::now().into();

        match existing_user {
            Some(user) => {
                let mut user_active: user::ActiveModel = user.into();
                user_active.uname = Set(account.uname.clone());
                user_active.session = Set(session_data);
                user_active.updated_at = Set(now);

                let updated_user = user_active
                    .update(&self.db)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to update user in database: {}", e))?;
                Ok(updated_user)
            }
            None => {
                // 创建新用户
                let user_active = user::ActiveModel {
                    id: ActiveValue::NotSet,
                    mid: Set(account.mid as i64),
                    uname: Set(account.uname.clone()),
                    session: Set(session_data),
                    created_at: Set(now.clone()),
                    updated_at: Set(now),
                };

                let new_user = user_active
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to insert user into database: {}", e))?;
                Ok(new_user)
            }
        }
    }

    pub async fn clean_session(&self, user: &user::Model) -> anyhow::Result<()> {
        tracing::trace!("Cleaning session for user {:?}", user);
        let mut user_active = user.clone().into_active_model();
        user_active.session = Set("".to_string());
        user_active
            .update(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to clean session in database: {}", e))?;
        Ok(())
    }

    pub async fn _get_user_by_id(&self, id: i64) -> anyhow::Result<Option<user::Model>> {
        let user = user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query user from database: {}", e))?;
        Ok(user)
    }

    pub async fn get_user_by_mid(&self, mid: i64) -> anyhow::Result<Option<user::Model>> {
        let user = user::Entity::find()
            .filter(user::Column::Mid.eq(mid))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query user from database: {}", e))?;
        Ok(user)
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
}
