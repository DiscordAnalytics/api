use mongodb::{
    Collection, Database,
    bson::{Bson, Document, doc},
    error::Result,
};

#[derive(Clone, Default)]
pub struct UpdateBuilder {
    inner: Document,
}

impl UpdateBuilder {
    pub fn add_to_set(mut self, document: Document) -> Self {
        let entry = self
            .inner
            .entry("$addToSet")
            .or_insert_with(|| Bson::Document(doc! {}));

        if let Some(doc) = entry.as_document_mut() {
            doc.extend(document);
        }

        self
    }

    pub fn inc(mut self, document: Document) -> Self {
        let entry = self
            .inner
            .entry("$inc")
            .or_insert_with(|| Bson::Document(doc! {}));

        if let Some(doc) = entry.as_document_mut() {
            doc.extend(document);
        }

        self
    }

    pub fn set(mut self, document: Document) -> Self {
        let entry = self
            .inner
            .entry("$set")
            .or_insert_with(|| Bson::Document(doc! {}));

        if let Some(doc) = entry.as_document_mut() {
            doc.extend(document);
        }

        self
    }

    pub fn build(self) -> Document {
        self.inner
    }
}

pub async fn ensure_collection<T>(db: &Database, name: &str) -> Result<Collection<T>>
where
    T: Sync + Send,
{
    let exists = db.list_collection_names().await?.iter().any(|n| n == name);

    if !exists {
        db.create_collection(name).await?;
    }

    Ok(db.collection::<T>(name))
}
