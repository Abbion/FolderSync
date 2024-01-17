use std::collections::HashMap;
use serde::Serialize;
use std::sync::Mutex;

#[derive(serde::Deserialize, Debug, Clone, Serialize)]
pub enum IntervalType {
    SECOND = 0,
    MINUTE = 1,
    HOUR = 2
}

pub trait ToIntervalType {
    fn to_interval_type(self) -> IntervalType;
}

impl ToIntervalType for i64 {
    fn to_interval_type(self) -> IntervalType {
        match self {
            0 => IntervalType::SECOND,
            1 => IntervalType::MINUTE,
            2 => IntervalType::HOUR,
            _ => IntervalType::SECOND,
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone, Serialize, PartialEq)]
pub enum SyncState {
    ENABLED = 0,
    DISABLED = 1,
    LOCKED = 2
}

pub trait ToSyncStateType {
    fn to_sync_state_type(self) -> SyncState;
}

impl ToSyncStateType for i64 {
    fn to_sync_state_type(self) -> SyncState {
        match self {
            0 => SyncState::ENABLED,
            1 => SyncState::DISABLED,
            2 => SyncState::LOCKED,
            _ => SyncState::LOCKED
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone, Serialize)]
pub struct SyncData {
    pub id: u64,
    pub from_path: String,
    pub to_path: String,
    pub interval_value: u64,
    pub interval_time: u64,
    pub interval_type: IntervalType,
    pub sync_state: SyncState
}

pub struct Database {
    pub sync_entries: Mutex<HashMap<u64, SyncData>>,
    pub next_id: Mutex<u64>,
    pub edited_id: Mutex<Option<u64>>,
    pub sql_connection: Mutex<sqlite::Connection>
}

impl Database {
    pub fn new() -> Database {
        Database{
            sync_entries: Mutex::new(HashMap::new()),
            next_id: Mutex::new(0),
            edited_id: Mutex::new(None),
            sql_connection : Mutex::new(sqlite::open("sync.db").unwrap())
        }
    }
}