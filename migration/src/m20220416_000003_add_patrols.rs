use entity::event;
use entity::patrol;
use entity::saved_voice_channel;
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220414_000003_add_patrols"
    }
}

#[async_trait::async_trait]
#[rustfmt::skip]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(event::Entity)
                .col(ColumnDef::new(event::Column::Id).integer().not_null().primary_key().auto_increment())
                .col(ColumnDef::new(event::Column::Start).date_time().not_null())
                .col(ColumnDef::new(event::Column::End).date_time().not_null())
                .col(ColumnDef::new(event::Column::Hosts).text().not_null())
                .to_owned(),
        ).await?;
        
        manager.create_table(
            Table::create()
                .table(saved_voice_channel::Entity)
                .col(ColumnDef::new(saved_voice_channel::Column::Id).integer().not_null().primary_key().auto_increment())
                .col(ColumnDef::new(saved_voice_channel::Column::ChannelId).big_unsigned().not_null())
                .col(ColumnDef::new(saved_voice_channel::Column::GuildId).big_unsigned().not_null())
                .col(ColumnDef::new(saved_voice_channel::Column::Name).string().not_null())
                .to_owned(),
        ).await?;

        manager.create_table(
            Table::create()
                .table(patrol::Entity)
                .col(ColumnDef::new(patrol::Column::Id).integer().not_null().primary_key().auto_increment())
                .col(ColumnDef::new(patrol::Column::OfficerId).big_unsigned().not_null())
                .col(ColumnDef::new(patrol::Column::MainChannelId).integer().not_null())
                .col(ColumnDef::new(event::Column::Start).date_time().not_null())
                .col(ColumnDef::new(event::Column::End).date_time().not_null())
                .col(ColumnDef::new(patrol::Column::EventId).integer().not_null())
                .to_owned(),
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("FK-event-patrol")
                .from(patrol::Entity, patrol::Column::EventId)
                .to(event::Entity, event::Column::Id)
                .on_delete(ForeignKeyAction::Restrict)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("FK-saved_voice_channel-main_channel")
                .from(patrol::Entity, patrol::Column::MainChannelId)
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
                .name("FK-event-patrol")
                .table(patrol::Entity)
                .to_owned()
        ).await?;

        manager.drop_foreign_key(
            ForeignKey::drop()
                .name("FK-saved_voice_channel-main_channel")
                .table(patrol::Entity)
                .to_owned()
        ).await?;

        manager.drop_table(
            sea_query::Table::drop()
                .table(patrol::Entity)
                .to_owned()
        ).await?;

        manager.drop_table(
            sea_query::Table::drop()
                .table(saved_voice_channel::Entity)
                .to_owned()
        ).await?;

        manager.drop_table(
            sea_query::Table::drop()
                .table(event::Entity)
                .to_owned()
        ).await?;

        Ok(())
    }
}
