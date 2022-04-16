use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "patrols")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub officer_id: u64,
    pub main_channel_id: i32,
    pub start: DateTime,
    pub end: DateTime,
    pub event_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::saved_voice_channel::Entity",
        from = "Column::MainChannelId",
        to = "super::saved_voice_channel::Column::Id"
    )]
    MainChannel,
    #[sea_orm(
        belongs_to = "super::officer::Entity",
        from = "Column::OfficerId",
        to = "super::officer::Column::Id"
    )]
    Officer,
    #[sea_orm(
        belongs_to = "super::event::Entity",
        from = "Column::EventId",
        to = "super::event::Column::Id"
    )]
    Event,
}

impl ActiveModelBehavior for ActiveModel {}
