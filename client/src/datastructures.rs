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

use anyhow::Result;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct BatteryStatus {
    health: String,
    percentage: i8,
    plugged: String,
    status: String,
    temperature: f32,
    current: i32,
}

pub enum BatteryChangerStatus {
    Charging,
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
}

impl From<&str> for BatteryStatus {
    fn from(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }
}

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
struct RawMessageList(Vec<RawMessage>);

pub struct Message {
    threadid: u64,
    read: bool,
    number: String,
    timestamp: i64,
    body: String,
}

pub fn convert_string_to_timestamp(s: &str) -> Result<i64> {
    Ok(NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?.timestamp())
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

#[derive(Clone, Debug)]
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
