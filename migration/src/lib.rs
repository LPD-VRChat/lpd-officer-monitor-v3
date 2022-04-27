pub use sea_schema::migration::prelude::*;

mod m20220412_000001_create_table;
mod m20220412_000002_extend_officer;
mod m20220416_000003_add_patrols;
mod m20220416_000004_fix_patrols;
mod m20220418_000005_fix_events;
mod m20220424_000006_add_patrol_voice;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220412_000001_create_table::Migration),
            Box::new(m20220412_000002_extend_officer::Migration),
            Box::new(m20220416_000003_add_patrols::Migration),
            Box::new(m20220416_000004_fix_patrols::Migration),
            Box::new(m20220418_000005_fix_events::Migration),
            Box::new(m20220424_000006_add_patrol_voice::Migration),
        ]
    }
}
