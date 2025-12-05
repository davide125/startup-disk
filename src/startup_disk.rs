// SPDX-License-Identifier: MIT

use asahi_bless::{BootCandidate, Volume};
use std::env;
use std::path::Path;
use thiserror::Error;

mod asahi;
mod mock;

#[derive(Debug, Error)]
pub enum StartupDiskError {
    #[error("asahi-bless error: {0:?}")]
    Asahi(asahi_bless::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("uuid parse error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("invalid boolean flag: {0}")]
    Bool(#[from] std::str::ParseBoolError),
    #[error("privileged helper failed: {0}")]
    Privileged(String),
    #[error("invalid helper invocation: {0}")]
    InvalidHelperInvocation(String),
}

pub type Result<T> = std::result::Result<T, StartupDiskError>;

impl From<asahi_bless::Error> for StartupDiskError {
    fn from(value: asahi_bless::Error) -> Self {
        StartupDiskError::Asahi(value)
    }
}

fn is_asahi() -> bool {
    Path::new("/proc/device-tree/chosen/asahi,system-fw-version").exists()
}

pub trait StartupDiskTrait {
    fn is_supported(&self) -> bool;
    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>>;
    fn get_boot_volume(&self, device: &str, next: bool) -> Result<BootCandidate>;
    fn set_boot_volume(&self, device: &str, cand: &BootCandidate, next: bool) -> Result<()>;
}

enum StartupDiskLibrary {
    AsahiBless(asahi::AsahiBlessLibrary),
    Mock(mock::MockLibrary),
}

impl StartupDiskTrait for StartupDiskLibrary {
    fn is_supported(&self) -> bool {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.is_supported(),
            StartupDiskLibrary::Mock(lib) => lib.is_supported(),
        }
    }

    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.get_boot_candidates(),
            StartupDiskLibrary::Mock(lib) => lib.get_boot_candidates(),
        }
    }
    fn get_boot_volume(&self, device: &str, next: bool) -> Result<BootCandidate> {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.get_boot_volume(device, next),
            StartupDiskLibrary::Mock(lib) => lib.get_boot_volume(device, next),
        }
    }
    fn set_boot_volume(&self, device: &str, cand: &BootCandidate, next: bool) -> Result<()> {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.set_boot_volume(device, cand, next),
            StartupDiskLibrary::Mock(lib) => lib.set_boot_volume(device, cand, next),
        }
    }
}

pub fn get_vg_name(vg: &[Volume]) -> &str {
    for v in vg {
        if v.is_system {
            return &v.name;
        }
    }
    &vg[0].name
}

pub fn startup_disk_library() -> &'static dyn StartupDiskTrait {
    let use_mock_library = if cfg!(debug_assertions) {
        env::var("USE_MOCK_LIBRARY").is_ok() || !is_asahi()
    } else {
        env::var("USE_MOCK_LIBRARY").is_ok()
    };

    // Create an instance of the chosen implementation
    let startup_disk_library: &dyn StartupDiskTrait = if use_mock_library {
        &StartupDiskLibrary::Mock(mock::MockLibrary)
    } else {
        &StartupDiskLibrary::AsahiBless(asahi::AsahiBlessLibrary)
    };

    startup_disk_library
}
