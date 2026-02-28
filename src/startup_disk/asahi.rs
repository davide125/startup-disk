// SPDX-License-Identifier: MIT

use asahi_bless::BootCandidate;

use crate::startup_disk::is_asahi;
use crate::startup_disk::Result;
use crate::startup_disk::StartupDiskTrait;
use crate::{privileged, startup_disk::StartupDiskError};
use nix::unistd::geteuid;

pub struct AsahiBlessLibrary;
impl StartupDiskTrait for AsahiBlessLibrary {
    fn is_supported(&self) -> bool {
        is_asahi()
    }

    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        if geteuid().is_root() {
            asahi_bless::get_boot_candidates().map_err(StartupDiskError::from)
        } else {
            privileged::get_boot_candidates()
        }
    }

    fn get_boot_volume(&self, device: &str, next: bool) -> Result<BootCandidate> {
        if geteuid().is_root() {
            asahi_bless::get_boot_volume(device, next).map_err(StartupDiskError::from)
        } else {
            privileged::get_boot_volume(device, next)
        }
    }

    fn set_boot_volume(&self, device: &str, cand: &BootCandidate, next: bool) -> Result<()> {
        if geteuid().is_root() {
            asahi_bless::set_boot_volume(device, cand, next).map_err(StartupDiskError::from)
        } else {
            privileged::set_boot_volume(device, cand, next)
        }
    }
}
