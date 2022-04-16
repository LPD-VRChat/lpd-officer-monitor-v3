use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "saved_voice_channels")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub channel_id: u64,
    pub guild_id: u64,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
