// use serde::{Deserialize, Serialize};
use serde::Deserialize;
use serde_json::Value;


use rust_ocpp::v1_6::messages::boot_notification::{
    BootNotificationRequest,
    BootNotificationResponse,
};
use rust_ocpp::v1_6::messages::heart_beat::{
    HeartbeatRequest,
    HeartbeatResponse,
};
use rust_ocpp::v1_6::messages::status_notification::{
    StatusNotificationRequest,
    StatusNotificationResponse,
};
use rust_ocpp::v1_6::types::{
    RegistrationStatus,
};


pub enum OcppMessageTypeId {
    Call = 2,
    CallResult = 3,
    CallError = 4,
}


#[derive(Deserialize, Debug)]
pub struct OcppCall {
    pub message_type_id: i32,
    pub unique_id: String,
    pub action: String,
    pub payload: Value,
}


fn on_boot_notification(payload: Value) -> Value {
    let payload: BootNotificationRequest = serde_json::from_value(payload).unwrap();
    let response = BootNotificationResponse {
        current_time: chrono::offset::Utc::now(),
        interval: 300,
        status: RegistrationStatus::Accepted,
    };
    serde_json::to_value(&response).unwrap()
}


fn on_heartbeat(payload: Value) -> Value {
    let payload: HeartbeatRequest = serde_json::from_value(payload).unwrap();
    let response = HeartbeatResponse {
        current_time: chrono::offset::Utc::now(),
    };
    serde_json::to_value(&response).unwrap()
}


fn on_status_notification(payload: Value) -> Value {
    let payload: StatusNotificationRequest = serde_json::from_value(payload).unwrap();
    let response = StatusNotificationResponse {};
    serde_json::to_value(&response).unwrap()
}


pub fn handle_request(action: &str, payload: Value) -> Result<Value, String> {
    match action {
        "BootNotification" => Ok(on_boot_notification(payload)),
        "Heartbeat" => Ok(on_heartbeat(payload)),
        "StatusNotification" => Ok(on_status_notification(payload)),
        action => {
            Err(format!("unknown action {}", action))
        },
    }
}
