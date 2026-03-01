// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::panic;

use asahi_bless::BootCandidate;
use gpt::disk::LogicalBlockSize;
use gpt::GptConfig;
use nix::unistd::geteuid;
use uuid::Uuid;

use crate::startup_disk::is_asahi;
use crate::startup_disk::Result;
use crate::startup_disk::StartupDiskTrait;
use crate::{privileged, startup_disk::StartupDiskError};

fn swap_uuid(u: &Uuid) -> Uuid {
    let (a, b, c, d) = u.as_fields();
    Uuid::from_fields(a.swap_bytes(), b.swap_bytes(), c.swap_bytes(), d)
}

const APFS_PART_TYPE_GUID: &str = "7C3457EF-0000-11AA-AA11-00306543ECAC";

/* The icon is stored in /.VolumeIcon.icns in the Data volume of the APFS
container for a given partition. */
fn try_read_icon_from_volume(
    partition_path: &str,
    container_omap_root: u64,
    block_size: u32,
    vol_oid: u64,
) -> std::result::Result<Option<Vec<u8>>, String> {
    let file = File::open(partition_path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);

    let vol_block = apfs::omap::omap_lookup(&mut reader, container_omap_root, block_size, vol_oid)
        .map_err(|e| format!("{e}"))?;
    let vol_data =
        apfs::object::read_block(&mut reader, vol_block, block_size).map_err(|e| format!("{e}"))?;
    let vol_sb = apfs::superblock::ApfsSuperblock::parse(&vol_data).map_err(|e| format!("{e}"))?;

    let vol_omap_root = apfs::omap::read_omap_tree_root(&mut reader, vol_sb.omap_oid, block_size)
        .map_err(|e| format!("{e}"))?;
    let catalog_root =
        apfs::omap::omap_lookup(&mut reader, vol_omap_root, block_size, vol_sb.root_tree_oid)
            .map_err(|e| format!("{e}"))?;

    // Use the catalog to try and find /.VolumeIcon.icns
    let (_oid, inode) = match apfs::catalog::resolve_path(
        &mut reader,
        catalog_root,
        vol_omap_root,
        block_size,
        "/.VolumeIcon.icns",
    ) {
        Ok(result) => result,
        Err(apfs::ApfsError::FileNotFound(_)) => return Ok(None),
        Err(e) => return Err(format!("{e}")),
    };

    // Read the file's extents and data
    let extents = apfs::catalog::lookup_extents(
        &mut reader,
        catalog_root,
        vol_omap_root,
        block_size,
        inode.private_id,
    )
    .map_err(|e| format!("{e}"))?;

    let mut buf = Vec::new();
    apfs::extents::read_file_data(&mut reader, block_size, &extents, inode.size(), &mut buf)
        .map_err(|e| format!("{e}"))?;
    Ok(Some(buf))
}

pub fn read_volume_icons() -> Result<HashMap<Uuid, Vec<u8>>> {
    let mut icons = HashMap::new();

    let disk = GptConfig::new()
        .writable(false)
        .logical_block_size(LogicalBlockSize::Lb4096)
        .open("/dev/nvme0n1")
        .map_err(StartupDiskError::Io)?;

    for (i, v) in disk.partitions() {
        if v.part_type_guid.guid != APFS_PART_TYPE_GUID {
            continue;
        }

        let part_uuid = swap_uuid(&v.part_guid);
        let partition_path = format!("/dev/nvme0n1p{i}");

        // Read the container superblock to enumerate volume OIDs
        let vol_oids = match read_container_vol_oids(&partition_path) {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Warning: failed to read container on partition {i}: {e}");
                continue;
            }
        };

        /* We loop over each volume to try and find the one with the icon,
        catching errors as we go, as sealed volumes are expected to fail here */
        for &(vol_oid, omap_root, bs) in &vol_oids {
            let path = partition_path.clone();
            let result = panic::catch_unwind(move || {
                try_read_icon_from_volume(&path, omap_root, bs, vol_oid)
            });

            match result {
                Ok(Ok(Some(data))) => {
                    icons.insert(part_uuid, data);
                    break;
                }
                Ok(Ok(None)) => {}
                Ok(Err(e)) => {
                    eprintln!(
                        "Warning: failed to read icon from partition {i} volume {vol_oid:#x}: {e}"
                    );
                }
                Err(_) => {
                    eprintln!("Warning: partition {i} volume {vol_oid:#x}: apfs parser panicked");
                }
            }
        }
    }

    Ok(icons)
}

fn read_container_vol_oids(
    partition_path: &str,
) -> std::result::Result<Vec<(u64, u64, u32)>, String> {
    let file = File::open(partition_path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);

    let nxsb = apfs::superblock::read_nxsb(&mut reader).map_err(|e| format!("read nxsb: {e}"))?;
    let nxsb = apfs::superblock::find_latest_nxsb(&mut reader, &nxsb)
        .map_err(|e| format!("find latest nxsb: {e}"))?;
    let block_size = nxsb.block_size;

    let container_omap_root =
        apfs::omap::read_omap_tree_root(&mut reader, nxsb.omap_oid, block_size)
            .map_err(|e| format!("read omap: {e}"))?;

    Ok(nxsb
        .fs_oids
        .iter()
        .filter(|&&oid| oid != 0)
        .map(|&oid| (oid, container_omap_root, block_size))
        .collect())
}

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

    fn get_volume_icons(&self) -> Result<HashMap<Uuid, Vec<u8>>> {
        if geteuid().is_root() {
            read_volume_icons()
        } else {
            privileged::get_volume_icons()
        }
    }
}
