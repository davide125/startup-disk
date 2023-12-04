use asahi_bless::BootCandidate;
use asahi_bless::Error;
use rand::Rng;
use std::env;
use std::path::Path;
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

struct AsahiBlessLibrary;
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

struct MockLibrary;
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

enum StartupDiskLibrary {
    AsahiBless(AsahiBlessLibrary),
    Mock(MockLibrary),
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
    let use_mock_library = env::var("USE_MOCK_LIBRARY").is_ok();
    // Create an instance of the chosen implementation
    let startup_disk_library: &dyn StartupDiskTrait = if use_mock_library {
        &StartupDiskLibrary::Mock(MockLibrary)
    } else {
        &StartupDiskLibrary::AsahiBless(AsahiBlessLibrary)
    };

    startup_disk_library
}
