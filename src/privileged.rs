// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use adw::glib;
use asahi_bless::{BootCandidate, Volume};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::startup_disk::{Result, StartupDiskError};

pub const HELPER_FLAG: &str = "--privileged-helper";

#[derive(Serialize, Deserialize)]
struct SerializableVolume {
    name: String,
    is_system: bool,
}

#[derive(Serialize, Deserialize)]
struct SerializableBootCandidate {
    vg_uuid: String,
    volumes: Vec<SerializableVolume>,
    part_uuid: String,
}

impl From<Volume> for SerializableVolume {
    fn from(value: Volume) -> Self {
        SerializableVolume {
            name: value.name,
            is_system: value.is_system,
        }
    }
}

impl From<SerializableVolume> for Volume {
    fn from(value: SerializableVolume) -> Self {
        Volume {
            name: value.name,
            is_system: value.is_system,
        }
    }
}

impl From<BootCandidate> for SerializableBootCandidate {
    fn from(value: BootCandidate) -> Self {
        SerializableBootCandidate::from(&value)
    }
}

impl From<&BootCandidate> for SerializableBootCandidate {
    fn from(value: &BootCandidate) -> Self {
        SerializableBootCandidate {
            vg_uuid: value.vg_uuid.to_string(),
            volumes: value
                .volumes
                .iter()
                .map(|volume| SerializableVolume {
                    name: volume.name.clone(),
                    is_system: volume.is_system,
                })
                .collect(),
            part_uuid: value.part_uuid.to_string(),
        }
    }
}

impl TryFrom<SerializableBootCandidate> for BootCandidate {
    type Error = StartupDiskError;

    fn try_from(value: SerializableBootCandidate) -> Result<Self> {
        Ok(BootCandidate {
            vg_uuid: Uuid::parse_str(&value.vg_uuid)?,
            volumes: value.volumes.into_iter().map(Into::into).collect(),
            part_uuid: Uuid::parse_str(&value.part_uuid)?,
        })
    }
}

/// Returns Some(exit_code) when running as the privileged helper. The main entrypoint calls this early so in order to skip initializing GTK.
pub fn maybe_handle_privileged_invocation() -> Option<glib::ExitCode> {
    let mut args = env::args();
    let _exe = args.next();
    if args.next().as_deref() != Some(HELPER_FLAG) {
        return None;
    }

    let remaining: Vec<String> = args.collect();
    Some(handle_privileged_invocation(&remaining))
}

pub fn get_boot_candidates() -> Result<Vec<BootCandidate>> {
    let output = run_helper(&["get-boot-candidates"], None)?;
    let serialized: Vec<SerializableBootCandidate> = serde_json::from_str(&output)?;
    serialized
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>>>()
}

pub fn get_boot_volume(device: &str, next: bool) -> Result<BootCandidate> {
    let output = run_helper(
        &["get-boot-volume", device, &next.to_string()],
        None::<&[u8]>,
    )?;
    let serialized: SerializableBootCandidate = serde_json::from_str(&output)?;
    serialized.try_into()
}

pub fn set_boot_volume(device: &str, cand: &BootCandidate, next: bool) -> Result<()> {
    let payload = serde_json::to_vec(&SerializableBootCandidate::from(cand))?;
    run_helper(
        &["set-boot-volume", device, &next.to_string()],
        Some(payload.as_slice()),
    )?;
    Ok(())
}

pub fn get_volume_icons() -> Result<HashMap<Uuid, Vec<u8>>> {
    let output = run_helper(&["get-volume-icons"], None)?;
    let serialized: HashMap<String, Vec<u8>> = serde_json::from_str(&output)?;
    serialized
        .into_iter()
        .map(|(k, v)| Ok((Uuid::parse_str(&k)?, v)))
        .collect::<Result<HashMap<_, _>>>()
}

pub fn handle_privileged_invocation(args: &[String]) -> glib::ExitCode {
    match dispatch_privileged_command(args) {
        Ok(()) => glib::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            glib::ExitCode::FAILURE
        }
    }
}

fn dispatch_privileged_command(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("get-boot-candidates") => {
            let boot_candidates = asahi_bless::get_boot_candidates()?;
            let serializable: Vec<SerializableBootCandidate> = boot_candidates
                .into_iter()
                .map(SerializableBootCandidate::from)
                .collect();
            serde_json::to_writer(std::io::stdout(), &serializable)?;
        }
        Some("get-boot-volume") => {
            let device = args
                .get(1)
                .ok_or_else(|| {
                    StartupDiskError::InvalidHelperInvocation(
                        "device must be provided for get-boot-volume".into(),
                    )
                })?
                .as_str();
            let next = args
                .get(2)
                .ok_or_else(|| {
                    StartupDiskError::InvalidHelperInvocation(
                        "next flag must be provided for get-boot-volume".into(),
                    )
                })?
                .parse::<bool>()?;
            let boot_volume = asahi_bless::get_boot_volume(device, next)?;
            let serializable = SerializableBootCandidate::from(boot_volume);
            serde_json::to_writer(std::io::stdout(), &serializable)?;
        }
        Some("set-boot-volume") => {
            let device = args
                .get(1)
                .ok_or_else(|| {
                    StartupDiskError::InvalidHelperInvocation(
                        "device must be provided for set-boot-volume".into(),
                    )
                })?
                .as_str();
            let next = args
                .get(2)
                .ok_or_else(|| {
                    StartupDiskError::InvalidHelperInvocation(
                        "next flag must be provided for set-boot-volume".into(),
                    )
                })?
                .parse::<bool>()?;
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            let candidate: SerializableBootCandidate = serde_json::from_str(&buffer)?;
            let candidate: BootCandidate = candidate.try_into()?;
            asahi_bless::set_boot_volume(device, &candidate, next)?;
        }
        Some("get-volume-icons") => {
            let icons = crate::startup_disk::asahi::read_volume_icons()?;
            let serializable: HashMap<String, Vec<u8>> = icons
                .into_iter()
                .map(|(uuid, data)| (uuid.to_string(), data))
                .collect();
            serde_json::to_writer(std::io::stdout(), &serializable)?;
        }
        _ => {
            return Err(StartupDiskError::InvalidHelperInvocation(
                "unknown privileged command".into(),
            ))
        }
    }

    // Make sure the output is flushed before exiting.
    std::io::stdout().flush()?;
    Ok(())
}

fn run_helper(args: &[&str], stdin_payload: Option<&[u8]>) -> Result<String> {
    let exe = env::current_exe()?;
    let mut command = Command::new("pkexec");
    command
        .arg(exe)
        .arg(HELPER_FLAG)
        .args(args)
        .stdin(if stdin_payload.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn()?;
    if let Some(payload) = stdin_payload {
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(payload)?;
        }
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(StartupDiskError::Privileged(stderr.trim().to_string()));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}
