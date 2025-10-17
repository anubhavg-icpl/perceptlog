// src/ocsf.rs - OCSF (Open Cybersecurity Schema Framework) event structures and builders
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// OCSF (Open Cybersecurity Schema Framework) event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfEvent {
    pub metadata: OcsfMetadata,
    pub category_uid: i32,
    pub category_name: String,
    pub class_uid: i32,
    pub class_name: String,
    pub time: i64,
    pub type_uid: i32,
    pub type_name: String,
    pub activity_id: i32,
    pub activity_name: String,
    pub status: String,
    pub status_id: i32,
    pub severity: String,
    pub severity_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<OcsfUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<OcsfActor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<OcsfService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_endpoint: Option<OcsfEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_endpoint: Option<OcsfEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_protocol_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logon_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logon_type_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logon_process: Option<OcsfProcess>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_remote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mfa: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cleartext: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observables: Option<Vec<OcsfObservable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmapped: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone_offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    pub version: String,
    pub product: OcsfProduct,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logged_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfProduct {
    pub vendor_name: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfUser {
    pub name: String,
    pub uid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfActor {
    pub user: OcsfUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfService {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfEndpoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfProcess {
    pub name: String,
    pub cmd_line: String,
    pub uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfObservable {
    pub name: String,
    #[serde(rename = "type")]
    pub observable_type: String,
    pub type_id: i32,
    pub value: String,
}

/// Builder for OCSF events to provide a fluent API
pub struct OcsfEventBuilder {
    event: OcsfEvent,
}

impl OcsfEventBuilder {
    /// Create a new OCSF event builder with default values
    pub fn new() -> Self {
        Self {
            event: OcsfEvent {
                metadata: OcsfMetadata {
                    uid: None,
                    version: "1.6.0".to_string(),
                    product: OcsfProduct {
                        vendor_name: "Linux".to_string(),
                        name: "Authentication Logs".to_string(),
                        version: "system".to_string(),
                    },
                    logged_time: None,
                    log_name: None,
                    log_provider: None,
                    event_code: None,
                    profiles: Some(vec!["host".to_string()]),
                    log_version: None,
                    log_level: None,
                    original_time: None,
                },
                category_uid: 0,
                category_name: String::new(),
                class_uid: 0,
                class_name: String::new(),
                time: 0,
                type_uid: 0,
                type_name: String::new(),
                activity_id: 0,
                activity_name: String::new(),
                status: String::new(),
                status_id: 0,
                severity: String::new(),
                severity_id: 0,
                user: None,
                actor: None,
                service: None,
                src_endpoint: None,
                dst_endpoint: None,
                auth_protocol: None,
                auth_protocol_id: None,
                logon_type: None,
                logon_type_id: None,
                logon_process: None,
                is_remote: None,
                is_mfa: None,
                is_cleartext: None,
                status_code: None,
                status_detail: None,
                message: None,
                raw_data: None,
                observables: None,
                unmapped: None,
                timezone_offset: None,
            },
        }
    }

    /// Set the metadata for the event
    pub fn with_metadata(mut self, metadata: OcsfMetadata) -> Self {
        self.event.metadata = metadata;
        self
    }

    /// Set the category information
    pub fn with_category(mut self, uid: i32, name: impl Into<String>) -> Self {
        self.event.category_uid = uid;
        self.event.category_name = name.into();
        self
    }

    /// Set the class information
    pub fn with_class(mut self, uid: i32, name: impl Into<String>) -> Self {
        self.event.class_uid = uid;
        self.event.class_name = name.into();
        self
    }

    /// Set the event time
    pub fn with_time(mut self, time: i64) -> Self {
        self.event.time = time;
        self
    }

    /// Set the type information
    pub fn with_type(mut self, uid: i32, name: impl Into<String>) -> Self {
        self.event.type_uid = uid;
        self.event.type_name = name.into();
        self
    }

    /// Set the activity information
    pub fn with_activity(mut self, id: i32, name: impl Into<String>) -> Self {
        self.event.activity_id = id;
        self.event.activity_name = name.into();
        self
    }

    /// Set the status information
    pub fn with_status(mut self, status: impl Into<String>, status_id: i32) -> Self {
        self.event.status = status.into();
        self.event.status_id = status_id;
        self
    }

    /// Set the severity information
    pub fn with_severity(mut self, severity: impl Into<String>, severity_id: i32) -> Self {
        self.event.severity = severity.into();
        self.event.severity_id = severity_id;
        self
    }

    /// Set the user information
    pub fn with_user(mut self, user: OcsfUser) -> Self {
        self.event.user = Some(user);
        self
    }

    /// Set the message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.event.message = Some(message.into());
        self
    }

    /// Build the final OCSF event
    pub fn build(self) -> OcsfEvent {
        self.event
    }
}

impl Default for OcsfEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

