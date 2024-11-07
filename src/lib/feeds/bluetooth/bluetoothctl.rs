use std::{io::BufRead, time::Duration};

use anyhow::anyhow;

const BLUETOOTHCTL: &str = "bluetoothctl";

#[derive(Debug, PartialEq)]
pub struct Info {
    // pub id: String,
    pub bat_pct: Option<u8>,
}

#[tracing::instrument(skip_all)]
pub fn devices(timeout: Duration) -> anyhow::Result<Vec<String>> {
    let out = crate::process::exec_with_timeout(
        BLUETOOTHCTL,
        &["--", "devices"],
        timeout,
    )?;
    parse_devices(&out[..])
}

#[tracing::instrument(skip_all)]
pub fn devices_connected(timeout: Duration) -> anyhow::Result<Vec<String>> {
    let out = crate::process::exec_with_timeout(
        BLUETOOTHCTL,
        &["--", "devices", "Connected"],
        timeout,
    )?;
    parse_devices(&out[..])
}

#[tracing::instrument(skip_all)]
fn parse_devices<Bytes: AsRef<[u8]>>(
    out: Bytes,
) -> anyhow::Result<Vec<String>> {
    let mut device_ids = Vec::new();
    for line_result in out.as_ref().lines() {
        let line = line_result?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        match &fields[..] {
            ["Device", id, _name @ ..] => {
                device_ids.push(id.to_string());
            }
            _ => {
                tracing::warn!(?line, "Unexpected line in output.");
            }
        }
    }
    Ok(device_ids)
}

#[tracing::instrument(skip_all)]
pub fn info(id: &str, timeout: Duration) -> anyhow::Result<Info> {
    let out = crate::process::exec_with_timeout(
        BLUETOOTHCTL,
        &["--", "info", id],
        timeout,
    )?;
    parse_info(&out[..])
}

#[tracing::instrument(skip_all)]
fn parse_info<Bytes: AsRef<[u8]>>(out: Bytes) -> anyhow::Result<Info> {
    let mut found_id = false;
    let mut bat_pct: Option<u8> = None;
    for line_result in out.as_ref().lines() {
        let line = line_result?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        match &fields[..] {
            ["Device", _id, ..] => {
                // Just format sanity check - we don't actually use the ID, so
                // no need to collect it.
                found_id = true;
            }
            ["Battery", "Percentage:", _some_code_in_hex, bat_pct_in_braces] =>
            {
                bat_pct = bat_pct_in_braces
                    .strip_prefix("(")
                    .and_then(|x| x.strip_suffix(")"))
                    .and_then(|pct_str| pct_str.parse::<u8>().ok());
                break;
            }
            _ => (),
        }
    }
    found_id
        .then(|| Info { bat_pct })
        .ok_or(anyhow!("Failed to parse bluetoothctl device info."))
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_devices_connected() {
        let id = "04:52:C7:0A:BD:56".to_string();
        let name = "Bose QuietComfort 35";
        let out = format!("Device {id} {name}");
        assert_eq!(vec![id], super::parse_devices(out).unwrap());
    }

    #[test]
    fn parse_info() {
        let id = "04:52:C7:0A:BD:56".to_string();
        let bat_pct = 90;
        let out = format!(
            "Device {id} (public)
	Name: Bose QuietComfort 35
	Alias: Bose QuietComfort 35
	Class: 0x00240418 (2360344)
	Icon: audio-headphones
	Paired: yes
	Bonded: yes
	Trusted: no
	Blocked: no
	Connected: yes
	LegacyPairing: no
	UUID: Vendor specific           (00000000-deca-fade-deca-deafdecacaff)
	UUID: Serial Port               (00001101-0000-1000-8000-00805f9b34fb)
	UUID: Headset                   (00001108-0000-1000-8000-00805f9b34fb)
	UUID: Audio Sink                (0000110b-0000-1000-8000-00805f9b34fb)
	UUID: A/V Remote Control Target (0000110c-0000-1000-8000-00805f9b34fb)
	UUID: Advanced Audio Distribu.. (0000110d-0000-1000-8000-00805f9b34fb)
	UUID: A/V Remote Control        (0000110e-0000-1000-8000-00805f9b34fb)
	UUID: A/V Remote Control Cont.. (0000110f-0000-1000-8000-00805f9b34fb)
	UUID: Handsfree                 (0000111e-0000-1000-8000-00805f9b34fb)
	UUID: Phonebook Access Client   (0000112e-0000-1000-8000-00805f9b34fb)
	UUID: Phonebook Access          (00001130-0000-1000-8000-00805f9b34fb)
	UUID: Headset HS                (00001131-0000-1000-8000-00805f9b34fb)
	UUID: PnP Information           (00001200-0000-1000-8000-00805f9b34fb)
	UUID: Bose Corporation          (0000febe-0000-1000-8000-00805f9b34fb)
	Modalias: bluetooth:v009Ep400Cd0105
	Battery Percentage: 0x5a ({bat_pct})
"
        );
        let device_expected = super::Info {
            // id,
            bat_pct: Some(bat_pct),
        };
        let device_parsed = super::parse_info(out).unwrap();
        assert_eq!(device_expected, device_parsed);
    }
}
