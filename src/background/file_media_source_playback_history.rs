use chrono::Datelike;
use crate::entity;
use crate::entity::item::ActiveModelEx;
use crate::entity::items_progress_history::ModelEx;
use crate::entity::{item, items_progress_history};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono::Timelike;
use sea_orm::{DatabaseConnection, QueryOrder};
use std::time::Duration;
use chrono::naive::MIN_DATE;

#[derive(Clone)]
pub struct FileMediaSourcePlaybackHistory {
    db: DatabaseConnection,

    last_id: i32,
    last_session_key: String,
    last_updated: DateTime<Utc>,
}

impl FileMediaSourcePlaybackHistory {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            last_id: 0,
            last_session_key: String::new(),
            last_updated: min_date_time(),
        }
    }

    pub async fn filter(&self, query: &str) -> Vec<ModelEx> {
        let db = self.db.clone();
        let result = entity::items_progress_history::Entity::load()
            .with(item::Entity)
            .order_by_desc(item::Column::DateModified)
            .all(&db)
            .await;

        if let Ok(history_items) = result {
            return history_items;
        }
        vec![]
    }

    pub async fn update(&mut self, item: ActiveModelEx, random_session_key: &str, new_position: Duration) -> bool {
        let now = Utc::now();
        
        let id = item.id.clone().unwrap();
        if self.last_id == id && self.last_session_key == random_session_key && self.last_updated < now - Duration::from_secs(5) {
            return false;
        }

        let db = self.db.clone();
        let builder = items_progress_history::ActiveModel::builder()
            .set_session_key(random_session_key)
            .set_item(item)
            .set_position(duration_to_naive_time(new_position))
            .set_date_modified(now);

        let result = builder
            .save(&db)
            .await;

        if result.is_ok() {
            self.last_updated = now;
            return true;
        }
        false
    }




    /*
    pub async fn update_history(&self, item: MediaSourceItem, position: Duration) -> Option<entity::items_progress_history::ModelEx> {
        let db = self.db.clone();
        let result = entity::items_progress_history::Entity::load()
            .order_by_desc(item::Column::DateModified)
            .one(&db)
            .await;

        if let Ok(history_item_option) = result {
            return history_item_option;
        }
        None
    }


    pub async fn load_last_history_item(&self) -> Option<entity::items_progress_history::ModelEx> {
        let db = self.db.clone();
        let result = entity::items_progress_history::Entity::load()
            .order_by_desc(item::Column::DateModified)
            .one(&db)
            .await;

        if let Ok(history_item_option) = result {
            return history_item_option;
        }
        None
    }
    
    */
    
}


fn min_date_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(
        NaiveDate::MIN.year(),
        NaiveDate::MIN.month(),
        NaiveDate::MIN.day(),
        0, 0, 0
    ).unwrap()
}

fn duration_to_naive_time(d: Duration) -> NaiveTime {
    // Wrap around at 24h if duration is longer than a day
    let secs = d.as_secs() % 86_400;
    let nanos = d.subsec_nanos();

    NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nanos)
        .expect("invalid time (out of range)")
}