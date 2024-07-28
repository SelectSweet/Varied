pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_account;
mod m20231110_015021_follow;
mod m20231110_054551_media;
mod m20231110_081234_session;
mod m20231110_081239_task;
mod m20240109_114821_collection;
mod m20240511_103551_biscuit_key;
mod m20240727_090310_tower_session;

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
            Box::new(m20240109_114821_collection::Migration),
            Box::new(m20240511_103551_biscuit_key::Migration),
            Box::new(m20240727_090310_tower_session::Migration),
        ]
    }
}
