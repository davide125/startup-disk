// SPDX-License-Identifier: MIT

mod asahi;
mod mock;

use asahi_bless::BootCandidate;
use asahi_bless::Error;
use std::env;
use std::path::Path;

type Result<T> = std::result::Result<T, Error>;

fn is_asahi() -> bool {
    Path::new("/proc/device-tree/chosen/asahi,system-fw-version").exists()
}

pub trait StartupDiskTrait {
    fn is_supported(&self) -> bool;
    fn needs_escalation(&self, method: &str) -> bool;
    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>>;
    fn get_boot_volume(&self, next: bool) -> Result<BootCandidate>;
    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()>;
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

    fn needs_escalation(&self, method: &str) -> bool {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.needs_escalation(method),
            StartupDiskLibrary::Mock(lib) => lib.needs_escalation(method),
        }
    }

    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.get_boot_candidates(),
            StartupDiskLibrary::Mock(lib) => lib.get_boot_candidates(),
        }
    }
    fn get_boot_volume(&self, next: bool) -> Result<BootCandidate> {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.get_boot_volume(next),
            StartupDiskLibrary::Mock(lib) => lib.get_boot_volume(next),
        }
    }
    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()> {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.set_boot_volume(cand, next),
            StartupDiskLibrary::Mock(lib) => lib.set_boot_volume(cand, next),
        }
    }
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
