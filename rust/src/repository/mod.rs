pub struct GameRepository {
    pub db: mongodb::Database,
}

impl GameRepository {
    pub fn new(db: mongodb::Database) -> Self {
        Self { db }
    }

    pub async fn get_user_balance(&self, username: &str) -> mongodb::error::Result<Option<crate::models::User>> {
        let collection = self.db.collection::<crate::models::User>("users");
        let filter = mongodb::bson::doc! { "username": username };
        collection.find_one(filter, None).await
    }

    pub async fn update_balance(&self, username: &str, amount: rust_decimal::Decimal) -> mongodb::error::Result<()> {
        let collection = self.db.collection::<crate::models::User>("users");
        let filter = mongodb::bson::doc! { "username": username };
        let update = mongodb::bson::doc! { "$inc": { "balance": amount.to_string() } };
        collection.update_one(filter, update, None).await?;
        Ok(())
    }
}
