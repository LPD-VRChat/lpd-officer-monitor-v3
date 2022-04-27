use entity::patrol;
use entity::patrol_voice;
use entity::saved_voice_channel;
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220424_000006_add_patrol_voice"
    }
}

#[async_trait::async_trait]
#[rustfmt::skip]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(patrol_voice::Entity)
                .col(ColumnDef::new(patrol_voice::Column::Id).integer().not_null().primary_key().auto_increment())
                .col(ColumnDef::new(patrol_voice::Column::PatrolId).integer().not_null())
                .col(ColumnDef::new(patrol_voice::Column::ChannelId).integer().not_null())
                .col(ColumnDef::new(patrol_voice::Column::Start).date_time().not_null())
                .col(ColumnDef::new(patrol_voice::Column::End).date_time().not_null())
                .to_owned(),
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("FK-patrol_voice-patrol")
                .from(patrol_voice::Entity, patrol_voice::Column::PatrolId)
                .to(patrol::Entity, patrol::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("FK-patrol_voice-channel")
                .from(patrol_voice::Entity, patrol_voice::Column::ChannelId)
                .to(saved_voice_channel::Entity, saved_voice_channel::Column::Id)
                .on_delete(ForeignKeyAction::Restrict)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(
            ForeignKey::drop()
                .name("FK-patrol_voice-patrol")
                .table(patrol_voice::Entity)
                .to_owned()
        ).await?;

        manager.drop_foreign_key(
            ForeignKey::drop()
                .name("FK-patrol_voice-channel")
                .table(patrol_voice::Entity)
                .to_owned()
        ).await?;

        manager.drop_table(
            sea_query::Table::drop()
                .table(patrol_voice::Entity)
                .to_owned()
        ).await?;

        Ok(())
    }
}
