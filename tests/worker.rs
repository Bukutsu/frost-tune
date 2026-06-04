// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use frost_tune::core::DeviceInfo;
use frost_tune::hardware::worker::{UsbWorker, WorkerStatus};

#[tokio::test]
async fn test_worker_new_and_status() {
    let worker = UsbWorker::new();
    let rx = worker.status();
    let status = rx.await;
    assert!(status.is_ok(), "Worker should respond to status request");
    let status = status.unwrap();
    assert!(!status.connected, "Worker should start disconnected");
    assert_eq!(status.generation, 0, "Initial generation should be 0");
}

#[test]
fn test_worker_status_structure() {
    let status = WorkerStatus {
        connected: true,
        physically_present: true,
        device: Some(DeviceInfo {
            vendor_id: 0x1234,
            product_id: 0x5678,
            path: "/dev/bus/usb/001/002".to_string(),
            manufacturer: Some("Test Manufacturer".to_string()),
            product_string: None,
        }),
        available_devices: vec![],
        backend_reset: false,
        generation: 42,
        fatal_error: None,
    };
    assert!(status.connected);
    assert!(status.physically_present);
    assert!(status.device.is_some());
    assert_eq!(status.generation, 42);
}

#[tokio::test]
async fn test_worker_connect_disconnect_cycle() {
    let worker = UsbWorker::new();
    let rx = worker.status();
    let initial_status = rx.await.expect("Should get initial status");
    assert!(!initial_status.connected);
    let rx = worker.disconnect();
    let _ = rx.await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let rx = worker.status();
    let status = rx.await.expect("Should get status after disconnect");
    assert!(
        !status.connected,
        "Should be disconnected after disconnect call"
    );
}
