pub use sea_orm_migration::prelude::*;

mod m20250601_000001_create_user;
mod m20250601_000002_create_table;
mod m20250601_000003_create_playlists;
mod m20250627_000001_add_user_permissions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250601_000001_create_user::Migration),
            Box::new(m20250601_000002_create_table::Migration),
            Box::new(m20250601_000003_create_playlists::Migration),
            Box::new(m20250627_000001_add_user_permissions::Migration),
        ]
    }
}
