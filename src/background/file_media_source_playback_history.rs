use crate::entity;
use crate::entity::items_progress_history::ModelEx;
use crate::entity::{item, items_json_metadata, items_metadata, items_progress_history};
use chrono::Datelike;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use sea_orm::ColumnTrait;
use sea_orm::{DatabaseConnection, QueryFilter, QueryOrder};
use std::time::Duration;
use media_source::media_source_history_item::MediaSourceHistoryItem;

#[derive(Clone)]
pub struct FileMediaSourcePlaybackHistory {
    db: DatabaseConnection,
}

impl FileMediaSourcePlaybackHistory {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
        }
    }
    
    pub async fn filter(&self, query: &str) -> Vec<entity::items_progress_history::ModelEx> {
        // https://github.com/SeaQL/sea-orm/blob/984827a6de82f965b41a1d7eb36852702eac8755/tests/partial_model_tests.rs
        let db = self.db.clone();
        let result = entity::items_progress_history::Entity::load()
            .filter(items_progress_history::Column::ItemId.eq(query))  // ← Filter here
            .order_by_desc(items_progress_history::Column::DateModified)
            .with(item::Entity)
            // .filter(items_progress_history::Entity)
            //             .filter(item::Column::FileId.eq(file_id_str.clone()))
            // .filter(item::Column::ItemId.eq(self.last_id))
            .all(&db)
            .await;

        if let Ok(history_items) = result {
            return history_items;
        }
        vec![]
    }


    pub async fn find_latest(&self, item_id: &str) -> Option<ModelEx> {
        println!("history->find_latest({})", item_id);

        // https://github.com/SeaQL/sea-orm/blob/984827a6de82f965b41a1d7eb36852702eac8755/tests/partial_model_tests.rs
        let db = self.db.clone();
        let result = if item_id == "" {
            items_progress_history::Entity::load()
                .order_by_desc(items_progress_history::Column::DateModified)
                .with(item::Entity)
                .one(&db)
                .await
        } else {
            items_progress_history::Entity::load()
                .filter(items_progress_history::Column::ItemId.eq(item_id))  // ← Filter here
                .order_by_desc(items_progress_history::Column::DateModified)
                .with(item::Entity)
                .one(&db)
                .await
        };

        if let Ok(history_item) = result {
            if let Some(ex) = history_item.clone() {
                println!("found a history item id={}, pos={}", ex.item_id, ex.position);
            }
            return history_item;
        } else {
            println!("no history item found");
        }
        None
    }


    pub async fn update(&self, history_item: MediaSourceHistoryItem) -> bool {
        let db = self.db.clone();
        let item_id = history_item.item.id;

        println!("Update history item: {:?}", item_id.clone());

        let latest_history_item_option = self.find_latest(&item_id).await;

        if latest_history_item_option.clone().is_some() {
            let debug_item = latest_history_item_option.clone().unwrap();
            println!(" => A history item exists");
            println!(" => id:{}, item_id:{} session_key:{}", debug_item.id, debug_item.item_id, debug_item.session_key);

            if debug_item.session_key != history_item.session_key.to_string() {
                println!("session_key FAILED: {} != {}", debug_item.session_key, history_item.session_key.to_string())
            } else {
                println!("session_key MATCHED: {} != {}", debug_item.session_key, history_item.session_key.to_string())
            }
        }



        let upsert_item_option = if let Some(latest_history_item) = latest_history_item_option
            && latest_history_item.session_key == history_item.session_key.to_string() {

            Some(
                items_progress_history::ActiveModelEx::from(latest_history_item)
                    .set_date_modified(Utc::now())
                    .set_position(duration_to_naive_time(history_item.position)
                    )
            )
        } else {
            let now = Utc::now();
            println!(" => create a new history item");

            let item_result = item::Entity::load()
                .filter(item::Column::Id.eq(&item_id))
                .with(items_metadata::Entity)
                .with(items_json_metadata::Entity)
                .one(&db)
                .await;

            if let Ok(Some(item)) = item_result {
                println!(" => We have a media item");

                Some(
                    items_progress_history::ActiveModel::builder()
                        .set_session_key(history_item.session_key.to_string())
                        .set_item(item)
                        .set_position(duration_to_naive_time(history_item.position))
                        .set_date_modified(now)
                )
            } else {
                println!(" => No media item");

                None
            }
        };

        if let Some(upsert_item) = upsert_item_option {
            println!(" => We have an upsert item");

            let result = upsert_item
                .save(&db)
                .await;

            println!(" => The upsert resulted in {}", result.is_ok());
            result.is_ok()
        } else {
            println!(" => no upsert item :( ");

            false
        }


    }


    
}

/*
fn min_date_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(
        NaiveDate::MIN.year(),
        NaiveDate::MIN.month(),
        NaiveDate::MIN.day(),
        0, 0, 0
    ).unwrap()
}
*/
fn duration_to_naive_time(d: Duration) -> NaiveTime {
    // Wrap around at 24h if duration is longer than a day
    let secs = d.as_secs() % 86_400;
    let nanos = d.subsec_nanos();

    NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nanos)
        .expect("invalid time (out of range)")
}