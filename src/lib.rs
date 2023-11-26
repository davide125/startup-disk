use asahi_bless::BootCandidate;
use asahi_bless::Error;
use rand::Rng;
use std::env;
use uuid::Uuid;

type Result<T> = std::result::Result<T, Error>;

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

pub trait StartupDiskTrait {
    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>>;
    fn get_boot_volume(&self, next: bool) -> Result<BootCandidate>;
    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()>;
}

struct AsahiBlessLibrary;
impl StartupDiskTrait for AsahiBlessLibrary {
    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        // We need root for this to do anything useful
        sudo::escalate_if_needed().unwrap();
        asahi_bless::get_boot_candidates()
    }

    fn get_boot_volume(&self, next: bool) -> Result<BootCandidate> {
        // We need root for this to do anything useful
        sudo::escalate_if_needed().unwrap();
        asahi_bless::get_boot_volume(next)
    }

    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()> {
        // We need root for this to do anything useful
        sudo::escalate_if_needed().unwrap();
        asahi_bless::set_boot_volume(cand, next)
    }
}

struct MockLibrary;
impl StartupDiskTrait for MockLibrary {
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

enum StartupDiskLibrary {
    AsahiBless(AsahiBlessLibrary),
    Mock(MockLibrary),
}

impl StartupDiskTrait for StartupDiskLibrary {
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
    let use_mock_library = env::var("USE_MOCK_LIBRARY").is_ok();
    // Create an instance of the chosen implementation
    let startup_disk_library: &dyn StartupDiskTrait = if use_mock_library {
        &StartupDiskLibrary::Mock(MockLibrary)
    } else {
        &StartupDiskLibrary::AsahiBless(AsahiBlessLibrary)
    };

    startup_disk_library
}
