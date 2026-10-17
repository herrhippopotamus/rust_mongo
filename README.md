# Mongo Utility Library

I don't quite like the implementation of the MongoDB crate. I use this crate to simplify MongoDB operations
in my Rust repos where. The crate basically provides a thin wrapper around the MongoDB client that makes it
more ergonomical to insert, update and query documents.

```Rust
mongo
  .collection("portfolios")
  .upsert(&portfolio)
  .await?;
```
