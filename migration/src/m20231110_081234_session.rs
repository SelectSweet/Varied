use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(v_session::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(v_session::SessionId).text().not_null().primary_key())
                    .col(ColumnDef::new(v_session::Username).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(v_session::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum v_session {
    #[sea_orm(iden = "v_session")]
    Table,
    SessionId,
    Username
}
