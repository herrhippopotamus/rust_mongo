use anyhow::Result;
use mongodb::{Client, bson::Document, options::ClientOptions};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone, Debug)]
pub struct Credentials {
    pub uri: String,
    pub database: String,
}

#[derive(Deserialize, Serialize)]
struct Count {
    pub total: usize,
}
impl Credentials {
    pub fn from_envs() -> Self {
        let host = std::env::var("MONGO_HOST").unwrap_or_else(|_| "127.0.0.1:27017".to_string());
        let db = std::env::var("MONGO_DB").unwrap_or_else(|_| "trading".to_string());
        let user = std::env::var("MONGO_USER").unwrap_or_else(|_| "admin".to_string());
        let pwd = std::env::var("MONGO_PWD").unwrap_or_else(|_| "Nila".to_string());
        Credentials::new(&host, &db, &user, &pwd)
    }
    pub fn new(host: &str, db: &str, user: &str, pwd: &str) -> Self {
        let uri = format!("mongodb://{user}:{pwd}@{host}/{db}?authSource=admin");
        Self {
            uri,
            database: db.to_string(),
        }
    }
}

pub type MongoCredentials = Credentials;

pub trait Doc: Serialize + DeserializeOwned + Send + Sync {}
impl<T: Serialize + DeserializeOwned + Send + Sync> Doc for T {}

pub struct Mongo {
    credentials: Credentials,
    client: Client,
}
impl Credentials {
    pub async fn connect(self) -> Result<Mongo> {
        Mongo::new(self).await
    }
}
pub struct Collection<'a> {
    mongo: &'a Mongo,
    collection: &'a str,
}

impl Mongo {
    pub async fn new(credentials: Credentials) -> Result<Self> {
        let options = ClientOptions::parse(&credentials.uri).await?;
        let client = Client::with_options(options)?;
        Ok(Self {
            credentials,
            client,
        })
    }
    pub fn collection<'a>(&'a self, collection: &'a str) -> Collection<'a> {
        Collection {
            mongo: self,
            collection,
        }
    }
}
impl<'a> Collection<'a> {
    fn collection<T: Doc>(&self) -> mongodb::Collection<T> {
        self.mongo
            .client
            .database(&self.mongo.credentials.database)
            .collection(self.collection)
    }
    pub async fn update(&'a self, filter: &str, update: &str) -> Result<&'a Self> {
        let filter: Document = serde_json::from_str(filter).unwrap_or_default();
        let update: Document = serde_json::from_str(update).unwrap_or_default();
        self.collection::<Document>()
            .update_many(filter, update)
            .await?;
        Ok(self)
    }

    pub async fn insert_one<T: Doc>(&'a self, item: &T) -> Result<&'a Self> {
        self.collection::<T>().insert_one(item).await?;
        Ok(self)
    }

    pub async fn insert<T: Doc>(&'a self, items: &Vec<T>) -> Result<&'a Self> {
        self.collection::<T>().insert_many(items).await?;
        Ok(self)
    }

    pub async fn delete(&'a self, query: &str) -> Result<&'a Self> {
        let filter: Document = serde_json::from_str(query).unwrap_or_default();
        self.collection::<Document>().delete_many(filter).await?;
        Ok(self)
    }

    pub async fn query<T: Doc>(&self, query: &str) -> Result<Vec<T>> {
        let mut cursor = if query.trim().starts_with("[") {
            let pipeline: Vec<Document> = serde_json::from_str(query).unwrap_or_default();
            self.collection::<Document>().aggregate(pipeline).await?
        } else {
            let filter: Document = serde_json::from_str(query).unwrap_or_default();
            self.collection::<Document>().find(filter).await?
        };
        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(mongodb::bson::from_slice(cursor.current().as_bytes())?);
        }
        Ok(results)
    }
    pub async fn query_one<T: Doc>(&self, query: &str) -> Result<Option<T>> {
        let filter: Document = serde_json::from_str(query).unwrap_or_default();
        let doc = self.collection::<T>().find_one(filter).await?;
        Ok(doc)
    }
    pub async fn count(&self, filter: &str) -> Result<usize> {
        let pipeline = format!(
            "
            [
                {{\"$match\": {filter} }},
                {{\"$count\": \"total\" }}
            ]
        "
        );
        let count: Vec<Count> = self.query(&pipeline).await?;
        Ok(count
            .into_iter()
            .next()
            .map(|count| count.total)
            .unwrap_or(0))
    }
}
