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

mod datastructures;
mod database;
mod test;

use clap::{App, ArgMatches};
use datastructures::{BatteryStatus, CallLog, RawCallLogList};
use sqlx::Connection;
use tokio::process::Command;

async fn query_call_log() -> anyhow::Result<Vec<CallLog>> {
    let output = Command::new("termux-call-log")
        .output().await?.stdout;
    let output = String::from_utf8(output)?;
    let logs: RawCallLogList = serde_json::from_str(&output)?;
    Ok(logs.convert_to_vec())
}

async fn fetch_battery_status() -> anyhow::Result<BatteryStatus> {
    let output = Command::new("termux-battery-status")
        .output()
        .await?
        .stdout;
    let output = String::from_utf8(output)?;
    let status: BatteryStatus = serde_json::from_str(&output)?;
    Ok(status)
}

async fn async_main<'a>(matches: &ArgMatches<'a>) -> anyhow::Result<()> {
    let mut conn = sqlx::sqlite::SqliteConnection::connect("sms_client.db").await?;
    let first_run = sqlx::query(r#"SELECT name FROM sqlite_master WHERE type='table' AND "name"=?"#)
        .bind(database::current::META_TABLE)
        .fetch_all(&mut conn)
        .await?
        .is_empty();
    if first_run {
        sqlx::query(database::current::CREATE_STATEMENTS)
            .execute(&mut conn)
            .await?;
    }
    let battery_status = fetch_battery_status().await?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async_main(&matches))?;

    Ok(())
}
