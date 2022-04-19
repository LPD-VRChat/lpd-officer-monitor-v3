use entity::patrol;
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220418_000005_fix_events"
    }
}

#[async_trait::async_trait]
#[rustfmt::skip]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(patrol::Entity)
                .modify_column(ColumnDef::new(patrol::Column::EventId).integer())
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(patrol::Entity)
                .modify_column(ColumnDef::new(patrol::Column::EventId).integer().not_null())
                .to_owned()
        ).await?;

        Ok(())
    }
}
