/// All binary numbers are in network byte order.

use std::{fs::{self, File, Metadata}, os::unix::fs::{MetadataExt, PermissionsExt}, path::{Path, PathBuf}};


use sha1::{Digest, Sha1};

use crate::{error::RustGitError, hash::Hasher, object::GitObjectId};

fn as_u32_be(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    ((array[3] as u32) <<  0)
}

fn as_u16_be(array: &[u8; 2]) -> u16 {
    ((array[0] as u16) <<  8) +
    ((array[1] as u16) <<  0)
}

const DEFAULT_INDEX_NAME: &str = "index";

// Only supporting V2 for now.
#[derive(Debug)]
pub(crate) enum GitIndexVersion {
    V2,
}

impl GitIndexVersion {
    pub(crate) fn deserialize(bytes: &[u8; 4]) -> Result<GitIndexVersion, RustGitError> {
        match as_u32_be(bytes) {
            2 => Ok(GitIndexVersion::V2),
            other => Err(RustGitError::new(format!("unsupported index version {other}")))
        }
    }
}

#[derive(Debug)]
pub(crate) struct GitIndexHeader {
    version: GitIndexVersion,
    num_entries: u32,
}

impl GitIndexHeader {
    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndexHeader, RustGitError> {
        let bytes: [u8; 12] = bytes.try_into()?;
        // 4-byte signature:
        // The signature is { 'D', 'I', 'R', 'C' } (stands for "dircache")
        if &bytes[0..4] != b"DIRC" {
            return Err(RustGitError::new("missing header signature in index file"));
        }
        Ok(GitIndexHeader {
            version: GitIndexVersion::deserialize(&bytes[4..8].try_into()?)?,
            num_entries: as_u32_be(&bytes[8..12].try_into()?),
        })
    }
}

#[derive(Debug)]
pub(crate) enum GitIndexObjectType {
    RegularFile,
    SymbolicLink,
    GitLink,
}

#[derive(Debug)]
pub(crate) struct GitIndexTimestamp {
    seconds: u32,
    nanoseconds: u32,
}

impl GitIndexTimestamp {
    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndexTimestamp, RustGitError> {
        let seconds_bytes = bytes[0..4].try_into()?;
        let nanoseconds_bytes = bytes[4..8].try_into()?;

        Ok(GitIndexTimestamp {
            seconds: as_u32_be(seconds_bytes),
            nanoseconds: as_u32_be(nanoseconds_bytes),
        })
    }
}

#[derive(Debug)]
pub(crate) enum GitIndexUnixPermission {
    Permission0755,
    Permission0644,
    None,
}

#[derive(Debug)]
pub(crate) struct GitIndexMode {
    obj_type: GitIndexObjectType,
    unix_permission: GitIndexUnixPermission,
}

impl GitIndexMode {
    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndexMode, RustGitError> {
        let bytes: [u8; 4] = bytes.try_into()?;
        let (obj_type, unix_permission) =
            match bytes {
                [0b00000000, 0b00000000, 0b10000001, 0b11101101] => Ok((GitIndexObjectType::RegularFile, GitIndexUnixPermission::Permission0755)),
                [0b00000000, 0b00000000, 0b10000001, 0b10100100] => Ok((GitIndexObjectType::RegularFile, GitIndexUnixPermission::Permission0644)),

                [0b00000000, 0b00000000, 0b10100000, 0b00000000] => Ok((GitIndexObjectType::SymbolicLink, GitIndexUnixPermission::None)),

                [0b00000000, 0b00000000, 0b11100000, 0b00000000] => Ok((GitIndexObjectType::GitLink, GitIndexUnixPermission::None)),
                other => Err(RustGitError::new(format!("invalid mode '{other:#?}' in index entry")))
            }?;

        Ok(GitIndexMode {
            obj_type,
            unix_permission,
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum GitIndexStageFlag {
    RegularFileNoConflict,
    Base,
    Ours,
    Theirs,
}

#[derive(Debug)]
pub(crate) struct GitIndexFlags {
    assume_valid: bool,
    extended: bool,
    stage: GitIndexStageFlag,
    name_length: u16,
} 

impl GitIndexFlags {
    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndexFlags, RustGitError> {
        let bytes: &[u8; 2] = bytes.try_into()?;
        let assume_valid = bytes[0] & 0b10000000 != 0;
        let extended = bytes[0] & 0b01000000 != 0;
        let stage = 
            if bytes[0] & 0b00110000 != 0 {
                GitIndexStageFlag::Theirs
            } else if bytes[0] & 0b00100000 != 0 {
                GitIndexStageFlag::Ours
            } else if bytes[0] & 0b00010000 != 0 {
                GitIndexStageFlag::Base
            } else {
                GitIndexStageFlag::RegularFileNoConflict
            };
        let name_length = as_u16_be(&[bytes[0] & 0b00001111, bytes[1]]);

        Ok(GitIndexFlags {
            assume_valid,
            extended,
            stage,
            name_length,
        })
    }
}

/// Index entries are sorted in ascending order on the name field,
/// interpreted as a string of unsigned bytes (i.e. memcmp() order, no
/// localization, no special casing of directory separator '/'). Entries
/// with the same name are sorted by their stage field.
pub(crate) struct GitIndexEntry {
    last_metadata_update: GitIndexTimestamp,
    last_data_update: GitIndexTimestamp,
    dev: u32,
    ino: u32,
    mode: GitIndexMode,
    uid: u32,
    gid: u32,
    file_size: u32,
    name: GitObjectId,
    flags: GitIndexFlags,

    /// Entry path name (variable length) relative to top level directory
    /// (without leading slash). '/' is used as path separator. The special
    /// path components ".", ".." and ".git" (without quotes) are disallowed.
    /// Trailing slash is also disallowed.
    path_name: String,
}

impl GitIndexEntry {
    pub(crate) fn deserialize(bytes: &[u8]) -> Result<(GitIndexEntry, usize), RustGitError> {
        println!("start");
        let last_metadata_update = GitIndexTimestamp::deserialize(&bytes[0..8])?;
        println!("last_metadata_update: {last_metadata_update:#?}");
        let last_data_update = GitIndexTimestamp::deserialize(&bytes[8..16])?;
        println!("last_data_update: {last_data_update:#?}");
        let dev = as_u32_be(&bytes[16..20].try_into()?);
        println!("dev: {dev:#?}");
        let ino = as_u32_be(&bytes[20..24].try_into()?);
        println!("ino: {ino:#?}");
        let mode = GitIndexMode::deserialize(&bytes[24..28])?;
        println!("mode: {mode:#?}");
        let uid = as_u32_be(&bytes[28..32].try_into()?);
        println!("uid: {uid:#?}");
        let gid = as_u32_be(&bytes[32..36].try_into()?);
        println!("gid: {gid:#?}");
        let file_size = as_u32_be(&bytes[36..40].try_into()?);
        println!("file_size: {file_size:#?}");
        let name = GitObjectId::deserialize(&bytes[40..60])?;
        println!("name: {name:#?}");
        let flags = GitIndexFlags::deserialize(&bytes[60..62])?;
        println!("flags: {flags:#?}");

        let path_name_bytes = {
            if flags.name_length < 0xFFF {
                Ok(&bytes[62..(62+flags.name_length as usize)])
            } else {
                if let Some(null_index) = &bytes[62..].iter().position(|&b| b != b'\0') {
                    Ok(&bytes[62..null_index+1])
                } else {
                    Err(RustGitError::new("missing null byte for path name"))
                }
            }

        }?;

        let path_name = String::from_utf8(path_name_bytes.to_vec())?;
        println!("path_name: {path_name:?}");

        let processed_bytes = 62 + path_name_bytes.len();
        let padding = {
            let remainder = processed_bytes % 8;
            if remainder == 0 {
                8
            } else {
                8 - remainder
            }
        };

        let padded_processed_bytes = processed_bytes + padding;

        Ok((GitIndexEntry {
            last_metadata_update,
            last_data_update,
            dev,
            ino,
            mode,
            uid,
            gid,
            file_size,
            name,
            flags,
            path_name,
        }, padded_processed_bytes))
    }

    pub(crate) fn new(path: &str, metadata: &Metadata, obj_id: GitObjectId) -> GitIndexEntry {
        let (obj_type, unix_permission) =
            if metadata.is_symlink() {
                (GitIndexObjectType::SymbolicLink, GitIndexUnixPermission::None)
            } else if metadata.is_dir() {
                (GitIndexObjectType::GitLink, GitIndexUnixPermission::None)
            } else {
                // If file is executable by owner, we set 755, otherwise 644.
                let unix_permission = 
                    if metadata.permissions().mode() | 0b0100 != 0 {
                        GitIndexUnixPermission::Permission0755
                    } else {
                        GitIndexUnixPermission::Permission0644
                    };
                (GitIndexObjectType::RegularFile, unix_permission)
            };


        GitIndexEntry {
            last_metadata_update:  GitIndexTimestamp {
                seconds: metadata.ctime() as u32,
                nanoseconds: metadata.ctime_nsec() as u32,
            },
            last_data_update:  GitIndexTimestamp {
                seconds: metadata.mtime() as u32,
                nanoseconds: metadata.mtime_nsec() as u32,
            },
            dev: metadata.dev() as u32,
            ino: metadata.ino() as u32,
            uid: metadata.uid() as u32,
            gid: metadata.gid() as u32,
            file_size: metadata.size() as u32,
            name: obj_id,
            mode: GitIndexMode {
                obj_type,
                unix_permission,
            },
            flags: GitIndexFlags {
                assume_valid: false,
                extended: false,
                stage: GitIndexStageFlag::RegularFileNoConflict,
                name_length: 0,
            },
            path_name: String::from(path),
        }
    }
}

pub(crate) struct GitIndex {
    header: GitIndexHeader,
    entries: Vec<GitIndexEntry>,
    checksum: GitObjectId,
}

impl Default for GitIndex {
    fn default() -> Self {
        Self { 
            header: GitIndexHeader {
                version: GitIndexVersion::V2,
                num_entries: 0,
            },
            entries: vec![],
            checksum: GitObjectId::new(String::from("")),
        }
    }
}

impl GitIndex {
    // Reference for Git index binary format: https://git-scm.com/docs/index-format
    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndex, RustGitError> {
        let header = GitIndexHeader::deserialize(&bytes[0..12])?;

        let mut entries = Vec::new();
        let mut entry_start = 12;
        for _ in 0..header.num_entries {
            let (entry , processed_bytes) = GitIndexEntry::deserialize(&bytes[entry_start..])?;
            entries.push(entry);
            entry_start += processed_bytes;
        }

        // Skipping extensions for now.

        // Checksum will always be last 20 bytes 
        // TODO: use separate SHA type
        let checksum = GitObjectId::deserialize(&bytes[bytes.len()-20..])?;
        println!("checksum: {checksum:?}");
        let mut hasher = Sha1::new();
        hasher.update(&bytes[..bytes.len()-20]);
        let hash = hasher.finalize();
        let computed_checksum = GitObjectId::deserialize(&hash)?;

        if checksum != computed_checksum {
            return Err(RustGitError::new(format!("Index checksum {checksum} doesn't match computed hash {computed_checksum}")));
        }

        Ok(GitIndex {
            header,
            entries,
            checksum,
        })
    }

    /// Loads the index from the provided git directory.
    pub(crate) fn open(git_dir: &Path) -> Result<GitIndex, RustGitError> {
        let index_file_path: PathBuf = git_dir.join(DEFAULT_INDEX_NAME);

        if !index_file_path.exists() {
            return Ok(GitIndex::default());
        }

        let index_file_bytes = fs::read(index_file_path)?;

        let git_index: GitIndex = GitIndex::deserialize(&index_file_bytes)?;

        Ok(git_index)
    }

    pub(crate) fn write(&mut self, git_dir: &Path) -> Result<(), RustGitError> {
        let mut index_file = File::create(&git_dir.join(DEFAULT_INDEX_NAME))?;

        // TODO: custom serialize
        // let serialized = bincode::serialize(&self)?;

        // index_file.write(&serialized)?;

        Ok(())
    }

    pub(crate) fn add(&mut self, index_entry: GitIndexEntry) {
        // Replace existing index entry with same path & stage.
        let existing_entry = self.entries.iter_mut().find(|item| item.path_name == index_entry.path_name && item.flags.stage == index_entry.flags.stage);

        match existing_entry {
            Some(existing) => *existing = index_entry,
            None => self.entries.push(index_entry)
        }
    }

    pub(crate) fn try_find_by_path(&self, path: &str) -> Option<&GitIndexEntry> {
        // TODO: make this more efficient
        self.entries.iter().find(|item| item.path_name == path)
    }
}
