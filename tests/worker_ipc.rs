// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT
#![cfg(target_os = "linux")]

use frost_tune::error::{AppError, ErrorKind};
use frost_tune::hardware::helper_ipc::{HelperRequest, HelperResponse, IPC_VERSION};

#[test]
fn test_ipc_version_handshake() {
    let request = HelperRequest::Version;
    let serialized = serde_json::to_string(&request).expect("Failed to serialize version request");
    let deserialized: HelperRequest =
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    match deserialized {
        HelperRequest::Version => (),
        _ => panic!("Deserialized to wrong type"),
    }
    let response = HelperResponse::Version {
        version: IPC_VERSION.to_string(),
    };
    let serialized =
        serde_json::to_string(&response).expect("Failed to serialize version response");
    let deserialized: HelperResponse =
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    match deserialized {
        HelperResponse::Version { version } => {
            assert_eq!(version, IPC_VERSION);
        }
        _ => panic!("Deserialized to wrong type"),
    }
}

#[test]
fn test_ipc_error_handling() {
    let error = AppError::new(ErrorKind::DeviceLost, "Test error").with_context("test context");
    let response = HelperResponse::Error {
        error: error.clone(),
    };
    let serialized = serde_json::to_string(&response).expect("Failed to serialize error response");
    let deserialized: HelperResponse =
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    match deserialized {
        HelperResponse::Error { error: e } => {
            assert_eq!(e.message, "Test error");
            assert_eq!(e.kind, ErrorKind::DeviceLost);
        }
        _ => panic!("Deserialized to wrong type"),
    }
}

#[test]
fn test_ipc_request_serialization() {
    let requests = vec![
        HelperRequest::Connect { device: None },
        HelperRequest::Disconnect,
        HelperRequest::Status,
        HelperRequest::Version,
        HelperRequest::Shutdown,
        HelperRequest::PullPeq { strict: true },
        HelperRequest::PullPeq { strict: false },
    ];
    for req in requests {
        let serialized = serde_json::to_string(&req).expect("Failed to serialize request");
        let _deserialized: HelperRequest = serde_json::from_str(&serialized)
            .unwrap_or_else(|_| panic!("Failed to deserialize request: {:?}", req));
    }
}

#[test]
fn test_ipc_response_serialization() {
    let responses: Vec<HelperResponse> = vec![
        HelperResponse::Connected { device: None },
        HelperResponse::Disconnected,
        HelperResponse::Status {
            connected: false,
            physically_present: false,
            device: None,
        },
        HelperResponse::Version {
            version: "1.0.0".to_string(),
        },
        HelperResponse::Pulled {
            data: serde_json::json!({}),
        },
        HelperResponse::Pushed {
            data: serde_json::json!({}),
        },
        HelperResponse::Ok,
    ];
    for resp in responses {
        let serialized = serde_json::to_string(&resp).expect("Failed to serialize response");
        let _deserialized: HelperResponse = serde_json::from_str(&serialized)
            .unwrap_or_else(|_| panic!("Failed to deserialize response: {:?}", resp));
    }
}

#[test]
fn test_ipc_version_mismatch_detection() {
    let current_version = IPC_VERSION;
    let old_version = "0.9.0";
    assert_ne!(
        current_version, old_version,
        "Versions should differ for mismatch test"
    );
    let response = HelperResponse::Version {
        version: old_version.to_string(),
    };
    match response {
        HelperResponse::Version { version } => {
            assert_eq!(version, old_version);
            assert_ne!(version, IPC_VERSION);
        }
        _ => panic!("Wrong response type"),
    }
}

#[test]
fn test_ipc_request_response_roundtrip() {
    use frost_tune::hardware::helper_ipc::{IpcRequest, IpcResponse};

    let req = IpcRequest {
        id: 42,
        auth: "test-token".to_string(),
        payload: HelperRequest::Version,
    };
    let serialized_req = serde_json::to_string(&req).expect("Failed to serialize IpcRequest");
    let deserialized_req: IpcRequest =
        serde_json::from_str(&serialized_req).expect("Failed to deserialize IpcRequest");
    assert_eq!(deserialized_req.id, 42);
    match deserialized_req.payload {
        HelperRequest::Version => {}
        _ => panic!("Expected HelperRequest::Version"),
    }

    let resp = IpcResponse {
        id: 100,
        payload: HelperResponse::Version {
            version: IPC_VERSION.to_string(),
        },
    };
    let serialized_resp = serde_json::to_string(&resp).expect("Failed to serialize IpcResponse");
    let deserialized_resp: IpcResponse =
        serde_json::from_str(&serialized_resp).expect("Failed to deserialize IpcResponse");
    assert_eq!(deserialized_resp.id, 100);
    match deserialized_resp.payload {
        HelperResponse::Version { version } => {
            assert_eq!(version, IPC_VERSION);
        }
        _ => panic!("Expected HelperResponse::Version"),
    }
}
