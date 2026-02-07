use sea_orm::DatabaseConnection;

pub struct PlaybackHistory {
    db: DatabaseConnection,
}

impl PlaybackHistory {
    
    pub fn new(db: DatabaseConnection) -> Self {
        Self {db}
    }
    
    pub async fn restore_player() {
        
    }
    
    
    
}