// SPDX-License-Identifier: MIT

use asahi_bless::BootCandidate;

use rand::Rng;
use uuid::Uuid;

use crate::startup_disk::Result;
use crate::startup_disk::StartupDiskTrait;

fn generate_random_strings(num_strings: usize, max_length: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();

    (0..num_strings)
        .map(|_| {
            let string_length = rng.gen_range(1..=max_length);
            (0..string_length)
                .map(|_| rng.gen_range(b'a'..=b'z') as char)
                .collect()
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
                vol_names: generate_random_strings(2, 10),
                part_uuid: Uuid::new_v4(),
            },
            BootCandidate {
                vg_uuid: Uuid::new_v4(),
                vol_names: generate_random_strings(2, 10),
                part_uuid: Uuid::new_v4(),
            },
            BootCandidate {
                vg_uuid: Uuid::new_v4(),
                vol_names: generate_random_strings(2, 10),
                part_uuid: Uuid::new_v4(),
            },
        ];
        Ok(cands)
    }

    fn get_boot_volume(&self, _next: bool) -> Result<BootCandidate> {
        Ok(BootCandidate {
            vg_uuid: Uuid::new_v4(),
            vol_names: Vec::new(),
            part_uuid: Uuid::new_v4(),
        })
    }

    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()> {
        println!(
            "Setting boot volume: {} {}",
            cand.vol_names.join(", "),
            next
        );
        Ok(())
    }
}
