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

#[cfg(test)]
mod test {
    use crate::datastructures::{
        convert_string_to_timestamp, RawCallLogList, RawDeviceInfo, SIMState,
    };

    #[test]
    #[should_panic]
    fn test_time_convert() {
        convert_string_to_timestamp("2021-08-23 24:58:40").unwrap();
    }

    #[test]
    fn test_parse_call_logs() {
        let s = r#"
        [
            {"name": "","phone_number": "911","type": "MISSED","date": "2021-07-24 19:49:25","duration": "00:35"},
            {"name": "","phone_number": "119","type": "MISSED","date": "2021-07-31 15:33:02","duration": "00:38"},
            {"name": "","phone_number": "110","type": "MISSED","date": "2021-08-10 09:17:45","duration": "00:37"},
            {"name": "","phone_number": "0237253888","type": "MISSED","date": "2021-08-11 11:34:31","duration": "00:38"},
            {"name": "","phone_number": "0235636688","type": "MISSED","date": "2021-08-20 10:38:48","duration": "00:36"}
        ]
        "#;

        let logs: RawCallLogList = serde_json::from_str(s).unwrap();

        assert_eq!(logs.len(), 5);
    }

    #[test]
    fn test_parse_device_info() {
        let s: &str = r#"
        {
            "data_enabled": "false","data_activity": "none","data_state": "disconnected","device_id": null,"device_software_version": "00",
            "phone_count": 2,"phone_type": "gsm","network_operator": "","network_operator_name": "","network_country_iso": "us",
            "network_type": "unknown","network_roaming": false,"sim_country_iso": "","sim_operator": "","sim_operator_name": "",
            "sim_serial_number": null,"sim_subscriber_id": null,"sim_state": "pin_required"
        }"#;
        let device_info: RawDeviceInfo = serde_json::from_str(s).unwrap();

        assert!(matches!(device_info.get_sim_state(), SIMState::Locked));
    }
}
