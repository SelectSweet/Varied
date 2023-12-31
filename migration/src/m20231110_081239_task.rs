use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(v_task::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(v_task::Id).text().not_null().primary_key())
                    .col(ColumnDef::new(v_task::Username).text().not_null())
                    .col(ColumnDef::new(v_task::Type).text().not_null())
                    .col(ColumnDef::new(v_task::Progress).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(v_task::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum v_task {
    #[sea_orm(iden = "v_task")]
    Table,
    Id,
    Username,
    Type,
    Progress
}
