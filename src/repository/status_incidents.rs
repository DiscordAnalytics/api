use bson::{oid::ObjectId, serialize_to_bson};
use futures::StreamExt;
use mongodb::{Collection, Database, IndexModel, bson::doc, error::Result};

use crate::{
    domain::models::{AnnouncementStatus, AnnouncementType, StatusIncident, StoredStatusIncident},
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

    pub async fn find_public_by_id(
        &self,
        bot_id: &str,
        incident_id: ObjectId,
    ) -> Result<Option<StatusIncident>> {
        self.collection
            .find_one(doc! {
                "_id": incident_id,
                "botId": bot_id,
                "isPublic": true,
            })
            .await
    }

    pub async fn find_public_incidents(
        &self,
        bot_id: &str,
        incident_type: Option<AnnouncementType>,
        status: Option<AnnouncementStatus>,
        limit: u32,
        cursor: Option<IncidentCursor>,
    ) -> Result<IncidentPage> {
        let mut filter = doc! {
            "botId": bot_id,
            "isPublic": true
        };

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
}
