pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_account;
mod m20231110_015021_follow;
mod m20231110_054551_media;
mod m20231110_081234_session;
mod m20231110_081239_task;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_account::Migration),
            Box::new(m20231110_015021_follow::Migration),
            Box::new(m20231110_054551_media::Migration),
            Box::new(m20231110_081234_session::Migration),
            Box::new(m20231110_081239_task::Migration),
        ]
    }
}
