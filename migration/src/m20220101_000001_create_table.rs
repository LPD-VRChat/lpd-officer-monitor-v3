use entity::officer;
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
#[rustfmt::skip]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(officer::Entity)
                    .col(ColumnDef::new(officer::Column::Id).big_unsigned().not_null().primary_key())
                    .col(ColumnDef::new(officer::Column::VrchatName).string().not_null())
                    .col(ColumnDef::new(officer::Column::VrchatId).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(officer::Entity)
                    .to_owned()
            )
            .await
    }
}
