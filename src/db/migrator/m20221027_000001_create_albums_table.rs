use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221027_000001_create_albums_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Albums table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Albums::Table)
                    .col(ColumnDef::new(Albums::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Albums::Title).string().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Albums table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Albums::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Albums {
    Table,
    Id,
    Title,
}
