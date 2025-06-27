use sea_orm::prelude::*;
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, Set,
};

use crate::core::entity::user;

#[derive(Clone)]
pub struct UserData {
    db: DatabaseConnection,
}

impl UserData {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn save_user_info(
        &self,
        mid: i64,
        uname: String,
        session_data: String,
    ) -> anyhow::Result<user::Model> {
        let existing_user = user::Entity::find()
            .filter(user::Column::Mid.eq(mid))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query database: {}", e))?;

        let now: sea_orm::prelude::DateTimeWithTimeZone = chrono::Utc::now().into();

        match existing_user {
            Some(user) => {
                let mut user_active: user::ActiveModel = user.into();
                user_active.uname = Set(uname);
                user_active.session = Set(session_data);
                user_active.updated_at = Set(now);

                let updated_user = user_active
                    .update(&self.db)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to update user in database: {}", e))?;
                Ok(updated_user)
            }
            None => {
                // 检查是否为第一个用户（数据库中没有其他用户）
                let user_count = user::Entity::find()
                    .count(&self.db)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to count users: {}", e))?;

                let is_first_user = user_count == 0;

                let user_active = user::ActiveModel {
                    id: ActiveValue::NotSet,
                    mid: Set(mid),
                    uname: Set(uname),
                    session: Set(session_data),
                    is_admin: Set(is_first_user), // 第一个用户自动成为管理员
                    can_stream: Set(is_first_user), // 只有第一个用户（管理员）默认有开播权限
                    is_disabled: Set(false),      // 默认不禁用
                    created_at: Set(now.clone()),
                    updated_at: Set(now),
                };

                let new_user = user_active
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to insert user into database: {}", e))?;

                if is_first_user {
                    tracing::info!(
                        "First user registered as admin with stream permissions: {} (mid: {})",
                        new_user.uname,
                        new_user.mid
                    );
                } else {
                    tracing::info!(
                        "New user registered without stream permissions: {} (mid: {})",
                        new_user.uname,
                        new_user.mid
                    );
                }

                Ok(new_user)
            }
        }
    }

    pub async fn clean_session(&self, user: &user::Model) -> anyhow::Result<()> {
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

    pub async fn list_all_users(&self) -> anyhow::Result<Vec<user::Model>> {
        let users = user::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query users from database: {}", e))?;
        Ok(users)
    }

    pub async fn update_user_permissions(
        &self,
        user_id: i64,
        is_admin: Option<bool>,
        can_stream: Option<bool>,
        is_disabled: Option<bool>,
    ) -> anyhow::Result<user::Model> {
        let user = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query user from database: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let mut user_active = user.into_active_model();
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();

        if let Some(admin) = is_admin {
            user_active.is_admin = Set(admin);
        }
        if let Some(stream) = can_stream {
            user_active.can_stream = Set(stream);
        }
        if let Some(disabled) = is_disabled {
            user_active.is_disabled = Set(disabled);
        }
        user_active.updated_at = Set(now);

        let updated_user = user_active
            .update(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update user permissions in database: {}", e))?;
        Ok(updated_user)
    }
}
