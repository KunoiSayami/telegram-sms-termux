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
#[allow(dead_code)]
pub mod v1 {

    pub const CREATE_STATEMENTS: &str = r#"
    CREATE TABLE "call_logs" (
        "identifier" TEXT NOT NULL,
        "timestamp" INTEGER NOT NULL,
        PRIMARY KEY("identifier")
    );


    CREATE TABLE "messages" (
        "identifier" TEXT NOT NULL,
        "timestamp" INTEGER NOT NULL,
        PRIMARY KEY("identifier")
    );

    CREATE TABLE "notifications" (
        "identifier" TEXT NOT NULL,
        "timestamp" INTEGER NOT NULL,
        PRIMARY KEY("identifier")
    );

    CREATE TABLE "client_meta" (
        "key"	TEXT NOT NULL,
        "value"	TEXT NOT NULL,
        PRIMARY KEY("key")
    );
    
    INSERT INTO "client_meta" VALUES ('version', '1');
    "#;


    pub const DROP_STATEMENTS: &str = r#"
    DROP TABLE "call_logs";
    DROP TABLE "messages";
    DROP TABLE "notifications";
    DROP TABLE "client_meta";
    "#;

    pub const VERSION: &str = "1";

    pub const META_TABLE: &str = "client_meta";
}

pub use v1 as current;
pub use v1::VERSION as CURRENT_VERSION;