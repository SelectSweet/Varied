use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(v_biscuitkey::Table)
                    .col(ColumnDef::new(v_biscuitkey::PrivateKey).binary().not_null().primary_key())
                    .col(ColumnDef::new(v_biscuitkey::Username).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(v_biscuitkey::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum v_biscuitkey {
    Table,
    Id,
    PrivateKey,
    Username,
}
