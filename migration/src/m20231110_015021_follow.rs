use sea_orm_migration::prelude::*;
use crate::m20220101_000001_create_account::v_account;

#[derive(DeriveMigrationName)]
pub struct Migration;

// CREATE TABLE public.v_follow (
//     v_follow_username text NOT NULL,
//     v_following_username text[],
//     properties jsonb[],
//     CONSTRAINT v_follow_fk FOREIGN KEY (v_follow_username) REFERENCES public.account(username),
//     UNIQUE(v_follow_username)
// )



#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(v_follow::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(v_account::Id).text().primary_key())
                    .col(ColumnDef::new(v_follow::follower).text().not_null())
                    .col(ColumnDef::new(v_follow::following).text())
                    .col(ColumnDef::new(v_follow::Properties).json())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(v_follow::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum v_follow {
    #[sea_orm(iden = "v_follow")]
    Table,
    Id,
    follower,
    following,
    Properties
}
