// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use std::time::{Duration, Instant};

use crate::core::DeviceInfo;
use crate::hardware::device_io::DiscoveryProvider;

pub struct DiscoveryManager {
    pub providers: Vec<Box<dyn DiscoveryProvider>>,
    pub last_physical_check: Instant,
    pub check_interval: Duration,
}

impl DiscoveryManager {
    pub fn new() -> Self {
        Self {
            providers: vec![Box::new(crate::hardware::hid::HidDiscoveryProvider)],
            last_physical_check: Instant::now(),
            check_interval: Duration::from_millis(1000),
        }
    }

    pub fn should_check(&self) -> bool {
        self.last_physical_check.elapsed() >= self.check_interval
    }

    pub fn list_devices(&self) -> Vec<DeviceInfo> {
        let mut all_devices = Vec::new();
        for provider in &self.providers {
            if let Ok(mut devices) = provider.list_devices() {
                all_devices.append(&mut devices);
            }
        }
        all_devices
    }

    pub fn check_physical_presence(&mut self, current_info: Option<&DeviceInfo>) -> bool {
        self.last_physical_check = Instant::now();

        if let Some(info) = current_info {
            for provider in &self.providers {
                if let Ok(devices) = provider.list_devices() {
                    if devices.iter().any(|d| d.path == info.path) {
                        return true;
                    }
                }
            }
            false
        } else {
            false
        }
    }
}

impl Default for DiscoveryManager {
    fn default() -> Self {
        Self::new()
    }
}
