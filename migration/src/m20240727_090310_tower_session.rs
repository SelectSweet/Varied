use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(v_towersession::Table)
                    .col(
                        ColumnDef::new(v_towersession::Key)
                            .binary()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(v_towersession::Username).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(v_towersession::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum v_towersession {
    Table,
    Key,
    Username,
}
