use futures::StreamExt;
use mongodb::{
    Collection, Database, IndexModel,
    bson::{DateTime, Document, doc, oid::ObjectId, serialize_to_bson},
    error::Result,
    options::ReturnDocument,
};

use crate::{
    domain::models::{
        AnnouncementStatus, AnnouncementType, IncidentUpdate, StatusIncident, StoredStatusIncident,
    },
    openapi::schemas::{IncidentCursor, IncidentPage},
    repository::common::ensure_collection,
    utils::constants::STATUS_INCIDENTS_COLLECTION,
};

#[derive(Clone)]
pub struct StatusIncidentsRepository {
    collection: Collection<StatusIncident>,
}

impl StatusIncidentsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        let collection = ensure_collection(db, STATUS_INCIDENTS_COLLECTION).await?;

        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! {
                        "botId": 1,
                        "isPublic": 1,
                        "createdAt": -1,
                        "_id": -1,
                    })
                    .build(),
            )
            .await?;

        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! {
                        "botId": 1,
                        "isPublic": 1,
                        "status": 1,
                        "createdAt": -1,
                        "_id": -1,
                    })
                    .build(),
            )
            .await?;

        Ok(Self { collection })
    }

    pub async fn create_incident(&self, incident: StatusIncident) -> Result<ObjectId> {
        let result = self.collection.insert_one(incident).await?;

        Ok(result
            .inserted_id
            .as_object_id()
            .expect("MongoDB should generate an ObjectId for the incident"))
    }

    pub async fn find_by_id(
        &self,
        bot_id: &str,
        incident_id: ObjectId,
        public_only: bool,
    ) -> Result<Option<StatusIncident>> {
        let mut filter = doc! {
            "_id": incident_id,
            "botId": bot_id,
        };

        if public_only {
            filter.insert("isPublic", true);
        }

        self.collection.find_one(filter).await
    }

    pub async fn find_incidents(
        &self,
        bot_id: &str,
        incident_type: Option<AnnouncementType>,
        status: Option<AnnouncementStatus>,
        limit: u32,
        cursor: Option<IncidentCursor>,
        public_only: bool,
    ) -> Result<IncidentPage> {
        let mut filter = doc! {
            "botId": bot_id,
        };

        if public_only {
            filter.insert("isPublic", true);
        }

        if let Some(incident_type) = incident_type {
            filter.insert("incidentType", serialize_to_bson(&incident_type)?);
        }

        if let Some(status) = status {
            filter.insert("status", serialize_to_bson(&status)?);
        }

        if let Some(cursor) = cursor {
            filter.insert(
                "$or",
                vec![
                    doc! {
                        "createdAt": { "$lt": cursor.created_at }
                    },
                    doc! {
                        "createdAt": cursor.created_at,
                        "_id": { "$lt": cursor.id }
                    },
                ],
            );
        }

        let collection = self.collection.clone_with_type::<StoredStatusIncident>();

        let mut cursor = collection
            .find(filter)
            .sort(doc! {
                "createdAt": -1,
                "_id": -1
            })
            .limit(i64::from(limit) + 1)
            .await?;

        let mut incidents = Vec::with_capacity(limit as usize + 1);

        while let Some(result) = cursor.next().await {
            incidents.push(result?);
        }

        let has_more = incidents.len() > limit as usize;

        if has_more {
            incidents.pop();
        }

        Ok(IncidentPage {
            incidents,
            has_more,
        })
    }

    pub async fn update_incident(
        &self,
        bot_id: &str,
        incident_id: ObjectId,
        update: Document,
    ) -> Result<Option<StatusIncident>> {
        self.collection
            .find_one_and_update(
                doc! {
                  "_id": incident_id,
                  "botId": bot_id,
                },
                doc! {
                  "$set": update
                },
            )
            .return_document(ReturnDocument::After)
            .await
    }

    pub async fn create_update(
        &self,
        bot_id: &str,
        incident_id: ObjectId,
        expected_status: AnnouncementStatus,
        update: IncidentUpdate,
        next_status: AnnouncementStatus,
    ) -> Result<Option<StatusIncident>> {
        let now = DateTime::now();

        let mut set = doc! {
            "status": serialize_to_bson(&next_status)?,
            "updatedAt": DateTime::now(),
        };

        match next_status {
            AnnouncementStatus::Cancelled
            | AnnouncementStatus::Resolved
            | AnnouncementStatus::Completed => {
                set.insert("resolvedAt", now);
            }
            AnnouncementStatus::InProgress => {
                set.insert("startedAt", now);
            }
            _ => {}
        }

        self.collection
            .find_one_and_update(
                doc! {
                    "_id": incident_id,
                    "botId": bot_id,
                    "status": serialize_to_bson(&expected_status)?,
                },
                doc! {
                    "$set": set,
                    "$push": {
                        "updates": serialize_to_bson(&update)?
                    }
                },
            )
            .return_document(ReturnDocument::After)
            .await
    }
}
