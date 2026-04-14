use mongodb::bson::{Bson, Document, doc};

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
            doc.extend(document.into_iter());
        }

        self
    }

    pub fn inc(mut self, document: Document) -> Self {
        let entry = self
            .inner
            .entry("$inc")
            .or_insert_with(|| Bson::Document(doc! {}));

        if let Some(doc) = entry.as_document_mut() {
            doc.extend(document.into_iter());
        }

        self
    }

    pub fn set(mut self, document: Document) -> Self {
        let entry = self
            .inner
            .entry("$set")
            .or_insert_with(|| Bson::Document(doc! {}));

        if let Some(doc) = entry.as_document_mut() {
            doc.extend(document.into_iter());
        }

        self
    }

    pub fn build(self) -> Document {
        self.inner
    }
}
