pub use sea_schema::migration::prelude::*;

mod m20220412_000001_create_table;
mod m20220412_000002_extend_officer;
mod m20220416_000003_add_patrols;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220412_000001_create_table::Migration),
            Box::new(m20220412_000002_extend_officer::Migration),
            Box::new(m20220416_000003_add_patrols::Migration),
        ]
    }
}
