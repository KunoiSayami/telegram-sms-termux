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

use datastructures::RawCallLogList;
use tokio::process::Command;


#[tokio::main]
async fn main() {
    let cmd = Command::new("termux-call-log")
        .output().await.unwrap().stdout;
    let s = String::from_utf8(cmd).unwrap();
    let log: RawCallLogList = serde_json::from_str(&s).unwrap();
    println!("{:#?}", &log);
}
