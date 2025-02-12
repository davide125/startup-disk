// SPDX-License-Identifier: MIT

use asahi_bless::{BootCandidate, Volume};

use rand::Rng;
use uuid::Uuid;

use crate::startup_disk::get_vg_name;
use crate::startup_disk::Result;
use crate::startup_disk::StartupDiskTrait;

fn generate_random_volumes(
    num_volumes: usize,
    max_length: usize,
    is_system: &[bool],
) -> Vec<Volume> {
    let mut rng = rand::rng();

    (0..num_volumes)
        .map(|n| {
            let string_length = rng.random_range(1..=max_length);
            let name = {
                (0..string_length)
                    .map(|_| rng.random_range(b'a'..=b'z') as char)
                    .collect()
            };
            Volume {
                name,
                is_system: is_system[n],
            }
        })
        .collect()
}

pub struct MockLibrary;
impl StartupDiskTrait for MockLibrary {
    fn is_supported(&self) -> bool {
        true
    }

    fn needs_escalation(&self, _method: &str) -> bool {
        false
    }

    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        let cands: Vec<BootCandidate> = vec![
            BootCandidate {
                vg_uuid: Uuid::new_v4(),
                volumes: generate_random_volumes(2, 10, &[true, false]),
                part_uuid: Uuid::new_v4(),
            },
            BootCandidate {
                vg_uuid: Uuid::new_v4(),
                volumes: generate_random_volumes(2, 10, &[false, true]),
                part_uuid: Uuid::new_v4(),
            },
            BootCandidate {
                vg_uuid: Uuid::new_v4(),
                volumes: generate_random_volumes(2, 10, &[false, false]),
                part_uuid: Uuid::new_v4(),
            },
        ];
        Ok(cands)
    }

    fn get_boot_volume(&self, _device: &str, _next: bool) -> Result<BootCandidate> {
        Ok(BootCandidate {
            vg_uuid: Uuid::new_v4(),
            volumes: Vec::new(),
            part_uuid: Uuid::new_v4(),
        })
    }

    fn set_boot_volume(&self, _device: &str, cand: &BootCandidate, next: bool) -> Result<()> {
        println!(
            "Setting boot volume: {} {}",
            get_vg_name(&cand.volumes),
            next
        );
        Ok(())
    }
}
