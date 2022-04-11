use super::schema::officers;

#[derive(Queryable, Debug)]
pub struct Officer {
    pub id: u64,
    pub vrchat_name: String,
    pub vrchat_id: String,
}

#[derive(Insertable)]
#[table_name = "officers"]
pub struct NewPost<'a> {
    pub id: u64,
    pub vrchat_name: &'a str,
    pub vrchat_id: &'a str,
}
