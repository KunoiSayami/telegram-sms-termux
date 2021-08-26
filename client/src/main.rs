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

mod database;
mod datastructures;
mod test;

use std::time::Duration;

use anyhow::Result;
use clap::{App, ArgMatches};
use datastructures::{
    BatteryStatus, CallLog, Identifier, Message, PermissionError, RawCallLogList, RawMessageList,
};
use sqlx::Connection;
use tokio::{process::Command, signal::ctrl_c, sync::mpsc};

use crate::datastructures::BatteryChangerStatus;

async fn fetch_sms() -> Result<Vec<Message>> {
    let output = Command::new("termux-message-list").output().await?.stdout;
    let output = String::from_utf8(output)?;
    let messages: RawMessageList = serde_json::from_str(&output)?;
    Ok(messages.convert_to_vec())
}

async fn fetch_call_log() -> Result<Vec<CallLog>> {
    let output = Command::new("termux-call-log").output().await?.stdout;
    let output = String::from_utf8(output)?;
    let logs: RawCallLogList = serde_json::from_str(&output)?;
    Ok(logs.convert_to_vec())
}

async fn fetch_battery_status() -> Result<BatteryStatus> {
    let output = Command::new("termux-battery-status").output().await?.stdout;
    let output = String::from_utf8(output)?;
    if output.contains("Error") {
        return Err(anyhow::Error::new(PermissionError::new()));
    }
    let status = BatteryStatus::from_str(&output)?;
    Ok(status)
}

async fn upstream(mut message_rx: mpsc::Receiver<InnerCommand>) -> Result<()> {
    loop {
        if let Ok(Some(cmd)) =
            tokio::time::timeout(Duration::from_secs(1), message_rx.recv()).await {
                match cmd {
                    InnerCommand::Message(msg) => {
                        todo!()
                    }
                    InnerCommand::Terminate => break,
                }
            }
    }
    Ok(())
}

async fn query_loop(
    message_tx: mpsc::Sender<InnerCommand>,
    mut terminate_rx: mpsc::Receiver<InnerCommand>,
) -> Result<()> {
    let mut battery_status = fetch_battery_status().await?.to_current_status();
    loop {
        let current_battery_status = fetch_battery_status().await?;

        match battery_status.not_equal(&current_battery_status) {
            datastructures::StatusDiff::ChargeStatus => {
                message_tx
                    .send(InnerCommand::Message(format!(
                        "[System Information]{}",
                        current_battery_status
                    )))
                    .await?;
                battery_status.update_charge_status(&current_battery_status)
            }
            datastructures::StatusDiff::Battery => {
                if current_battery_status.get_percentage() == 15 {
                    message_tx
                        .send(InnerCommand::Message(format!(
                            "[System Information]\n{}",
                            match current_battery_status.get_changer_status() {
                                BatteryChangerStatus::Charging => "Battery is low.",
                                BatteryChangerStatus::Discharging =>
                                    "Battery has been charged to a safe level.",
                            }
                        )))
                        .await?;
                }
                battery_status.update_charge_status(&current_battery_status)
            }
            datastructures::StatusDiff::Equal => todo!(),
        }

        if let Ok(Some(cmd)) =
            tokio::time::timeout(Duration::from_secs(1), terminate_rx.recv()).await
        {
            match cmd {
                InnerCommand::Terminate => break,
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
enum InnerCommand {
    Message(String),
    Terminate,
}

async fn async_main<'a>(matches: &ArgMatches<'a>) -> Result<()> {
    let mut conn = sqlx::sqlite::SqliteConnection::connect("sms_client.db").await?;

    let first_run =
        sqlx::query(r#"SELECT name FROM sqlite_master WHERE type='table' AND "name"=?"#)
            .bind(database::current::META_TABLE)
            .fetch_all(&mut conn)
            .await?
            .is_empty();

    if first_run {
        let call_logs = fetch_call_log().await;
        let messages = fetch_sms().await;
        if let Err(ref e) = call_logs {
            if e.is::<PermissionError>() {
                log::error!("Fetch call log error: {}", e);
            } else {
                log::error!("Unknown error in fetch call log: {}", e);
            }
            return Err(anyhow::Error::msg("Exit due to error show above"));
        }
        if let Err(ref e) = messages {
            if e.is::<PermissionError>() {
                log::error!("Fetch call log error: {}", e);
            } else {
                log::error!("Unknown error in fetch sms list: {}", e);
            }
            return Err(anyhow::Error::msg("Exit due to error show above"));
        }
        sqlx::query(database::current::CREATE_STATEMENTS)
            .execute(&mut conn)
            .await?;
        for call_log in call_logs? {
            sqlx::query(r#"INSERT INTO "call_logs" (?, ?)"#)
                .bind(call_log.get_identifier())
                .bind(call_log.get_timestamp())
                .execute(&mut conn)
                .await?;
        }
        for sms in messages? {
            sqlx::query(r#"INSERT INTO "messages" (?, ?)"#)
                .bind(sms.get_identifier())
                .bind(sms.get_timestamp())
                .execute(&mut conn)
                .await?;
        }
    }
    let (msg_tx, msg_rx) = mpsc::channel(1024);
    let (query_tx, query_rx) = mpsc::channel(1024);
    let query_task = tokio::task::spawn(query_loop(msg_tx.clone(), query_rx));
    let upstream_task = tokio::task::spawn(upstream(msg_rx));

    loop {
        if let Ok(Ok(_)) = tokio::time::timeout(Duration::from_millis(500), ctrl_c()).await {
            break;
        }
    }
    query_tx.send(InnerCommand::Terminate).await?;
    msg_tx.send(InnerCommand::Terminate).await?;
    query_task.await??;
    upstream_task.await??;
    Ok(())
}

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async_main(&matches))?;

    Ok(())
}
