use entity::officer;
use entity::patrol;
use entity::saved_voice_channel;
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220416_000004_fix_patrols"
    }
}

#[async_trait::async_trait]
#[rustfmt::skip]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_foreign_key(
            ForeignKey::create()
                .name("FK-patrol-officer")
                .from(patrol::Entity, patrol::Column::OfficerId)
                .to(officer::Entity, officer::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name("unique-discord-channel")
                .table(saved_voice_channel::Entity)
                .col(saved_voice_channel::Column::GuildId)
                .col(saved_voice_channel::Column::ChannelId)
                .unique()
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(
            ForeignKey::drop()
                .name("FK-patrol-officer")
                .table(patrol::Entity)
                .to_owned()
        ).await?;

        manager.drop_index(
            Index::drop()
                .name("unique-discord-channel")
                .table(saved_voice_channel::Entity)
                .to_owned()
        ).await?;

        Ok(())
    }
}
