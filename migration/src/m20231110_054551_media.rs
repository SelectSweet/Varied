use sea_orm_migration::prelude::*;

// id text NOT NULL,
//     publicid text NOT NULL,
//     title text NOT NULL,
//     v_mediatype text NOT NULL,
//     uploaded_at timestamp default (now() at time zone 'utc') NOT NULL,
//     username text NOT NULL, 
//     description text,
//     chapters json,
//     storagepathorurl text[],
//     properties json,
//     state text NOT NULL,
//     CONSTRAINT v_media_pk PRIMARY KEY (id),
//     UNIQUE (id),
//     UNIQUE (publicid)


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        let create_table = manager
            .create_table(
                Table::create()
                    .table(v_media::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(v_media::Id)
                            .text()
                            .unique_key()
                            .not_null()
                            .primary_key(),
                            
                    )
                    .col(ColumnDef::new(v_media::Publicid).text().not_null().unique_key())
                    .col(ColumnDef::new(v_media::Title).text().not_null())
                    .col(ColumnDef::new(v_media::mediatype).text().not_null())
                    .col(ColumnDef::new(v_media::Uploaded_at).date_time().not_null())
                    .col(ColumnDef::new(v_media::Username).text().not_null())
                    .col(ColumnDef::new(v_media::Description).text())
                    .col(ColumnDef::new(v_media::Chapters).json())
                    .col(ColumnDef::new(v_media::Storagepathorurl).array(table::ColumnType::Text))
                    .col(ColumnDef::new(v_media::PosterStoragepathorurl).array(table::ColumnType::Text))
                    .col(ColumnDef::new(v_media::Properties).json().not_null())
                    .col(ColumnDef::new(v_media::State).text().not_null())    
                    .to_owned()                    
            );
        create_table
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(v_media::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum v_media {
    #[sea_orm(iden = "v_media")]
    Table,
    Id,
    Publicid,
    Title,
    mediatype,
    Uploaded_at,
    Username, 
    Description,
    Chapters,
    Storagepathorurl,
    PosterStoragepathorurl,
    Properties,
    State,
}
