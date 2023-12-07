// SPDX-License-Identifier: MIT

use asahi_bless::BootCandidate;

use crate::startup_disk::is_asahi;
use crate::startup_disk::Result;
use crate::startup_disk::StartupDiskTrait;

pub struct AsahiBlessLibrary;
impl StartupDiskTrait for AsahiBlessLibrary {
    fn is_supported(&self) -> bool {
        is_asahi()
    }

    fn needs_escalation(&self, method: &str) -> bool {
        match method {
            "get_boot_candidates" => true,
            "get_boot_volume" => true,
            "set_boot_volume" => true,
            &_ => false,
        }
    }

    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        asahi_bless::get_boot_candidates()
    }

    fn get_boot_volume(&self, next: bool) -> Result<BootCandidate> {
        asahi_bless::get_boot_volume(next)
    }

    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()> {
        asahi_bless::set_boot_volume(cand, next)
    }
}
