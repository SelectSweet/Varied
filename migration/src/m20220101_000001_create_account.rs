use sea_orm_migration::prelude::*;


// CREATE TABLE public.Account
// (                              
//     id text NOT NULL,
//     public_id text NOT NULL,                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   
//     username text NOT NULL,
//     password text NOT NULL,
//     email text NOT NULL,
//     created_at timestamp default (now() at time zone 'utc'),
//     display_name text NOT NULL,
//     avatar text NOT NULL,
//     profile_metadata json,
//     description text,
//     PRIMARY KEY (id, created_at),
//     UNIQUE (id),
//     UNIQUE (username),
//     UNIQUE (email)
// )


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(v_account::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(v_account::Id)
                            .text()
                            .not_null()
                            .primary_key().unique_key(),
                            
                    )
                    .col(ColumnDef::new(v_account::Username).text().not_null().unique_key())
                    .col(ColumnDef::new(v_account::Password).text().not_null())
                    .col(ColumnDef::new(v_account::Email).text().not_null().unique_key())
                    .col(ColumnDef::new(v_account::Created_At).date_time().not_null())
                    .col(ColumnDef::new(v_account::Display_Name).text().not_null())
                    .col(ColumnDef::new(v_account::Avatar).text().not_null())
                    .col(ColumnDef::new(v_account::Profile_Metadata).json())
                    .col(ColumnDef::new(v_account::Description).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(v_account::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum v_account {
    #[sea_orm(iden = "v_account")]
    Table,
    Id,
    Username,
    Password,
    Email,
    Created_At,
    Display_Name,
    Avatar,
    Profile_Metadata,
    Description,
}
