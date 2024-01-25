use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(v_collection::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(v_collection::PublicID)
                            .string()
                            .not_null()
                            .primary_key()
                    )
                    .col(ColumnDef::new(v_collection::Title).string().not_null())
                    .col(ColumnDef::new(v_collection::Description).string())
                    .col(ColumnDef::new(v_collection::Type).string().not_null())
                    .col(ColumnDef::new(v_collection::State).string().not_null())
                    .col(ColumnDef::new(v_collection::IDs).json())
                    .col(ColumnDef::new(v_collection::Properties).json())
                    .col(ColumnDef::new(v_collection::Username).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(v_collection::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum v_collection {
    Table,
    PublicID,
    Title,
    Description,
    Type,
    State,
    IDs,
    Properties,
    Username
}
