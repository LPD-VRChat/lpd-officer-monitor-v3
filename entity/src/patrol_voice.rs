use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "patrol_voice_comms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub patrol_id: i32,
    pub channel_id: i32,
    pub start: DateTime,
    pub end: DateTime,
}

impl Related<super::patrol::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Patrol.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::saved_voice_channel::Entity",
        from = "Column::ChannelId",
        to = "super::saved_voice_channel::Column::Id"
    )]
    Channel,
    #[sea_orm(
        belongs_to = "super::patrol::Entity",
        from = "Column::PatrolId",
        to = "super::patrol::Column::Id"
    )]
    Patrol,
}

impl ActiveModelBehavior for ActiveModel {}
