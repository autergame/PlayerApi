pub use sea_orm_migration::prelude::*;

mod create_avatar_table;
mod create_favorite_table;
mod create_home_table;
mod create_login_table;
mod create_session_table;
mod create_userinfo_table;
mod create_watching_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(create_session_table::Migration),
            Box::new(create_login_table::Migration),
            Box::new(create_userinfo_table::Migration),
            Box::new(create_avatar_table::Migration),
            Box::new(create_favorite_table::Migration),
            Box::new(create_watching_table::Migration),
            Box::new(create_home_table::Migration),
        ]
    }
}
