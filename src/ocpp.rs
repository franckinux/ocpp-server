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


fn on_boot_notification(payload: BootNotificationRequest) -> BootNotificationResponse {
    BootNotificationResponse {
        current_time: chrono::offset::Utc::now(),
        interval: 300,
        status: RegistrationStatus::Accepted,
    }
}


fn on_heartbeat(payload: HeartbeatRequest) -> HeartbeatResponse {
    HeartbeatResponse {
        current_time: chrono::offset::Utc::now(),
    }
}


fn on_status_notification(payload: StatusNotificationRequest) -> StatusNotificationResponse {
    StatusNotificationResponse {}
}


pub fn handle_request(action: &str, payload: Value) -> Result<String, String> {
    match action {
        "BootNotification" => {
            let payload: BootNotificationRequest = serde_json::from_value(payload).unwrap();
            let response = on_boot_notification(payload);
            Ok(
                serde_json::to_string(&response).unwrap()
            )
        },
        "Heartbeat" => {
            let payload: HeartbeatRequest = serde_json::from_value(payload).unwrap();
            let response = on_heartbeat(payload);
            Ok(
                serde_json::to_string(&response).unwrap()
            )
        },
        "StatusNotification" => {
            let payload: StatusNotificationRequest = serde_json::from_value(payload).unwrap();
            let response = on_status_notification(payload);
            Ok(
                serde_json::to_string(&response).unwrap()
            )
        },
        action => {
            Err(format!("unknown action {}", action))
        },
    }
}
