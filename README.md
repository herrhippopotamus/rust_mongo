# Mongo Utility Library

I don't quite like the implementation of the MongoDB crate. I use this crate to simplify MongoDB operations
in my Rust repos where. The crate basically provides a thin wrapper around the MongoDB client that makes it
more ergonomical to insert, update and query documents.

```Rust
use mongo::client::{Id, Mongo};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Hippo {
    pub name: String,
    pub age: usize,
}
impl Id for Hippo {
    fn id(&self) -> &str {
        &self.name
    }
}

mongo
    .collection("hippos")
    .upsert(&hippo)
    .await?;

.mongo
    .collection("real_estate")
    .query(
        r#"[
            {"$match": {"deleted": {"$ne": true}}},
            {"$project": {"listing.images": 0}},
            {"$limit": 10000}
        ]"#,
    )
    .await?;

let coll = mongo.collection("real_estate");
if let Some(existing_listing) = coll.query_one::<ReviewedListing>(&filter).await? {
    listing.review = existing_listing.review;
}

coll
    .delete(&filter)
    .await?
    .insert(&vec![listing])
    .await?;
```
