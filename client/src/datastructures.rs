/*
 ** Copyright (C) 2021 KunoiSayami
 **
 ** This file is part of telegram-sms-termux and is released under
 ** the AGPL v3 License: https://www.gnu.org/licenses/agpl-3.0.txt
 **
 ** This program is free software: you can redistribute it and/or modify
 ** it under the terms of the GNU Affero General Public License as published by
 ** the Free Software Foundation, either version 3 of the License, or
 ** any later version.
 **
 ** This program is distributed in the hope that it will be useful,
 ** but WITHOUT ANY WARRANTY; without even the implied warranty of
 ** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 ** GNU Affero General Public License for more details.
 **
 ** You should have received a copy of the GNU Affero General Public License
 ** along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use std::{error::Error, fmt::Display};

use anyhow::Result;
use chrono::NaiveDateTime;
use serde::Deserialize;
use sha2::{digest::DynDigest, Digest, Sha256};

#[derive(Deserialize, Clone, Debug)]
pub struct Configure {
    upstream: String,
    applications: Option<Vec<String>>,
}

impl Configure {
    pub fn get_upstream(&self) -> &String {
        &self.upstream
    }
}

pub trait Identifier {
    fn get_timestamp(&self) -> i64;

    fn get_body(&self) -> String;

    fn get_identifier(&self) -> String {
        let mut sha256 = Sha256::new();
        let s = format!("{}{}", self.get_timestamp(), self.get_body());
        let bytes = s.as_bytes();
        DynDigest::update(&mut sha256, &bytes);
        let result = sha256.finalize();
        format!("{:x}", result)
    }
}

pub fn convert_string_to_timestamp(s: &str) -> Result<i64> {
    Ok(NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?.timestamp())
}

#[derive(Debug, Clone)]
pub struct PermissionError {}

impl Error for PermissionError {}

impl Display for PermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Permission denied, please give specify permission to termux:api app"
        )
    }
}

impl PermissionError {
    pub fn new() -> Self {
        Self {}
    }
}

pub mod battery {

    use anyhow::Result;
    use serde::Deserialize;

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct BatteryStatus {
        health: String,
        /// Real battery percentage
        percentage: i8,
        plugged: String,
        /// Battery changer status
        status: String,
        temperature: f32,
        current: i32,
    }

    impl std::fmt::Display for BatteryStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let status = self.get_changer_status();
            write!(
                f,
                "{}\nCurrent battery level: {}",
                match status {
                    BatteryChangerStatus::Charging => "Changer is connected.",
                    BatteryChangerStatus::Discharging => "Changer is disconnect.",
                },
                self.get_percentage()
            )
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum BatteryChangerStatus {
        /// Battery is charging
        Charging,
        /// Battery is discharging
        Discharging,
    }

    impl BatteryStatus {
        pub fn from_str(s: &str) -> Result<Self> {
            Ok(serde_json::from_str(s)?)
        }

        pub fn get_percentage(&self) -> i8 {
            self.percentage
        }

        pub fn get_changer_status(&self) -> BatteryChangerStatus {
            if self.status.to_lowercase().eq("charging") {
                BatteryChangerStatus::Charging
            } else {
                BatteryChangerStatus::Discharging
            }
        }

        pub fn to_current_status(&self) -> CurrentStatus {
            CurrentStatus::from(self)
        }
    }

    impl From<&str> for BatteryStatus {
        fn from(s: &str) -> Self {
            serde_json::from_str(s).unwrap()
        }
    }

    #[derive(Debug, Clone)]
    pub struct CurrentStatus {
        charge_status: BatteryChangerStatus,
        battery_level: i8,
    }

    impl CurrentStatus {
        pub fn update_charge_status(&mut self, status: &BatteryStatus) {
            self.charge_status = status.get_changer_status();
            self.battery_level = status.get_percentage();
        }

        pub fn not_equal(&self, status: &BatteryStatus) -> StatusDiff {
            if self.charge_status != status.get_changer_status() {
                return StatusDiff::ChargeStatus;
            } else if self.battery_level != status.get_percentage() {
                return StatusDiff::Battery;
            }
            StatusDiff::Equal
        }

        pub fn get_battery_level(&self) -> i8 {
            self.battery_level
        }

        pub fn get_changer_status(&self) -> BatteryChangerStatus {
            self.charge_status.clone()
        }
    }

    impl From<&BatteryStatus> for CurrentStatus {
        fn from(bs: &BatteryStatus) -> Self {
            Self {
                charge_status: bs.get_changer_status(),
                battery_level: bs.get_percentage(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum StatusDiff {
        ChargeStatus,
        Battery,
        Equal,
    }

    impl Default for CurrentStatus {
        fn default() -> Self {
            Self {
                charge_status: BatteryChangerStatus::Discharging,
                battery_level: Default::default(),
            }
        }
    }
}

pub mod sms {
    use super::{convert_string_to_timestamp, Identifier};
    use serde::Deserialize;

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct RawMessage {
        threadid: u64,
        #[serde(rename = "type")]
        message_type: String,
        read: bool,
        number: String,
        received: String,
        body: String,
    }

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct RawMessageList(Vec<RawMessage>);

    impl RawMessageList {
        pub fn convert_to_vec(&self) -> Vec<Message> {
            self.0
                .iter()
                .map(|element| Message::from(element))
                .collect()
        }
    }

    #[allow(dead_code)]
    #[derive(Clone, Debug)]
    pub struct Message {
        threadid: u64,
        read: bool,
        number: String,
        timestamp: i64,
        body: String,
    }

    impl Message {
        pub fn get_number(&self) -> &String {
            &self.number
        }

        pub fn get_content(&self) -> &String {
            &self.body
        }
    }

    impl From<&RawMessage> for Message {
        fn from(m: &RawMessage) -> Self {
            Self {
                threadid: m.threadid,
                read: m.read,
                number: m.number.clone(),
                timestamp: convert_string_to_timestamp(&m.received).unwrap(),
                body: m.body.clone(),
            }
        }
    }

    impl Identifier for Message {
        fn get_timestamp(&self) -> i64 {
            self.timestamp
        }

        fn get_body(&self) -> String {
            format!("{}{}", self.number, self.body)
        }
    }
}

pub mod call_log {
    use super::{convert_string_to_timestamp, Identifier};
    use serde::Deserialize;

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct RawCallLog {
        name: String,
        phone_number: String,
        #[serde(rename = "type")]
        log_type: String,
        date: String,
        duration: String,
    }

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct RawCallLogList(Vec<RawCallLog>);

    impl RawCallLogList {
        pub fn len(&self) -> usize {
            self.0.len()
        }

        pub fn convert_to_vec(&self) -> Vec<CallLog> {
            self.0
                .iter()
                .map(|element| CallLog::from(element))
                .collect()
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum CallLogType {
        INCOMING,
        OUTGOING,
        REJECTED,
        MISSED,
    }

    impl CallLogType {
        pub fn parse_type(t: &str) -> Self {
            match t {
                "MISSED" => Self::MISSED,
                "REJECTED" => Self::REJECTED,
                "OUTGOING" => Self::OUTGOING,
                "INCOMING" => Self::INCOMING,
                _ => unreachable!("SHOULD UPDATE CALL LOG TYPE: {}", t),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct CallLog {
        name: String,
        log_type: CallLogType,
        timestamp: i64,
        phone_number: String,
        duration: String,
    }

    impl From<&RawCallLog> for CallLog {
        fn from(l: &RawCallLog) -> Self {
            Self {
                name: l.name.clone(),
                log_type: CallLogType::parse_type(&l.log_type),
                timestamp: convert_string_to_timestamp(&l.date).unwrap(),
                phone_number: l.phone_number.clone(),
                duration: l.duration.clone(),
            }
        }
    }

    impl CallLog {
        pub fn get_log_type(&self) -> &CallLogType {
            &self.log_type
        }

        pub fn get_number(&self) -> &String {
            &self.phone_number
        }
    }

    impl From<&RawCallLogList> for Vec<CallLog> {
        fn from(l: &RawCallLogList) -> Self {
            l.convert_to_vec()
        }
    }

    impl Identifier for CallLog {
        fn get_timestamp(&self) -> i64 {
            self.timestamp
        }

        fn get_body(&self) -> String {
            self.phone_number.clone()
        }
    }
}

pub mod device_info {
    use serde::Deserialize;

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct RawDeviceInfo {
        data_enabled: String,
        data_activity: String,
        data_state: String,
        device_id: Option<String>,
        // IMEI
        device_software_version: String,
        phone_count: u8,
        phone_type: String,
        network_operator: String,
        network_operator_name: String,
        network_country_iso: String,
        network_type: String,
        network_roaming: bool,
        sim_country_iso: String,
        sim_operator: String,
        sim_operator_name: String,
        sim_serial_number: Option<String>,
        sim_subscriber_id: Option<String>,
        sim_state: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum SIMState {
        Ready,
        Locked,
        NotInsert,
        Unknown,
    }

    impl From<&str> for SIMState {
        fn from(s: &str) -> Self {
            match s {
                "ready" => Self::Ready,
                "pin_required" => Self::Locked,
                "absent" => Self::NotInsert,
                _ => Self::Unknown,
            }
        }
    }

    impl std::fmt::Display for SIMState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    SIMState::Ready => "ready",
                    SIMState::Locked => "locked",
                    SIMState::NotInsert => "not insert",
                    SIMState::Unknown => "unknown",
                }
            )
        }
    }

    impl RawDeviceInfo {
        pub fn get_sim_state(&self) -> SIMState {
            SIMState::from(self.sim_state.as_str())
        }
    }
}

pub mod notification {
    use serde::Deserialize;

    #[allow(dead_code, non_snake_case)]
    #[derive(Deserialize, Clone, Debug)]
    struct RawNotification {
        id: i64,
        tag: String,
        key: String,
        group: String,
        packageName: String,
        title: String,
        content: String,
        when: String,
    }

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    struct RawNotificationList(Vec<RawNotification>);
}

pub use battery::{BatteryChangerStatus, BatteryStatus, StatusDiff};
pub use call_log::{CallLog, CallLogType, RawCallLogList};
pub use device_info::{RawDeviceInfo, SIMState};
pub use sms::{Message, RawMessageList};
