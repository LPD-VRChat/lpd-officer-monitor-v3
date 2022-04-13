use entity::officer;
use sea_query::MySqlQueryBuilder;
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220412_000002_extend_officer"
    }
}

#[async_trait::async_trait]
#[rustfmt::skip]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(officer::Entity)
                .add_column(ColumnDef::new(officer::Column::StartedMonitoring).date_time().not_null())
                .to_owned()
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(officer::Entity)
                .add_column(ColumnDef::new(officer::Column::DeleteAt).date_time().default(0))
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(officer::Entity)
                .drop_column(officer::Column::StartedMonitoring)
                .to_owned()
        ).await?;
        
        manager.alter_table(
            Table::alter()
                .table(officer::Entity)
                .drop_column(officer::Column::DeleteAt)
                .to_owned()
        ).await?;

        Ok(())
    }
}
