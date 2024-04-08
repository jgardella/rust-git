/// All binary numbers are in network byte order.

use std::{fmt::Display, fs::{self, File, Metadata}, io::Write, os::unix::fs::{MetadataExt, PermissionsExt}, path::{Path, PathBuf}};
use sha1::{Digest, Sha1};
use crate::{error::RustGitError, hash::Hasher, object::GitObjectId};

const INDEX_HEADER_SIGNATURE: &[u8; 4] = b"DIRC";

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
#[derive(Debug, PartialEq)]
pub(crate) enum GitIndexVersion {
    V2,
}

impl GitIndexVersion {
    pub(crate) fn serialize(index_version: &GitIndexVersion) -> [u8; 4] {
        match index_version {
            GitIndexVersion::V2 => (2 as u32).to_be_bytes()
        }
    }

    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndexVersion, RustGitError> {
        let bytes: &[u8; 4] = bytes.try_into()?;
        match as_u32_be(bytes) {
            2 => Ok(GitIndexVersion::V2),
            other => Err(RustGitError::new(format!("unsupported index version {other}")))
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct GitIndexHeader {
    version: GitIndexVersion,
    num_entries: u32,
}

impl GitIndexHeader {
    pub(crate) fn serialize(header: &GitIndexHeader) -> [u8; 12] {
        let mut bytes: [u8; 12] = [0; 12];
        bytes[0..4].copy_from_slice(INDEX_HEADER_SIGNATURE);
        bytes[4..8].copy_from_slice(&GitIndexVersion::serialize(&header.version));
        bytes[8..12].copy_from_slice(&header.num_entries.to_be_bytes());

        bytes
    }

    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndexHeader, RustGitError> {
        let bytes: [u8; 12] = bytes.try_into()?;
        // 4-byte signature:
        // The signature is { 'D', 'I', 'R', 'C' } (stands for "dircache")
        if &bytes[0..4] != INDEX_HEADER_SIGNATURE {
            return Err(RustGitError::new("missing header signature in index file"));
        }
        Ok(GitIndexHeader {
            version: GitIndexVersion::deserialize(&bytes[4..8])?,
            num_entries: as_u32_be(&bytes[8..12].try_into()?),
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct GitIndexTimestamp {
    pub seconds: u32,
    pub nanoseconds: u32,
}

impl Display for GitIndexTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.seconds, self.nanoseconds)
    }
}

impl GitIndexTimestamp {
    pub(crate) fn serialize(timestamp: &GitIndexTimestamp) -> [u8; 8] {
        let mut bytes: [u8; 8] = [0; 8];
        bytes[0..4].copy_from_slice(&timestamp.seconds.to_be_bytes());
        bytes[4..8].copy_from_slice(&timestamp.nanoseconds.to_be_bytes());
        bytes
    }

    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndexTimestamp, RustGitError> {
        let bytes: &[u8; 8] = bytes.try_into()?;
        let seconds_bytes: &[u8; 4] = bytes[0..4].try_into()?;
        let nanoseconds_bytes: &[u8; 4] = bytes[4..8].try_into()?;

        Ok(GitIndexTimestamp {
            seconds: as_u32_be(seconds_bytes),
            nanoseconds: as_u32_be(nanoseconds_bytes),
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum GitIndexMode {
    RegularFile0755,
    RegularFile0644,
    SymbolicLink,
    GitLink,
}

impl Display for GitIndexMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitIndexMode::RegularFile0755 => write!(f, "{}", "100755"),
            GitIndexMode::RegularFile0644 => write!(f, "{}", "100644"),
            GitIndexMode::SymbolicLink => write!(f, "{}", "120000"),
            GitIndexMode::GitLink => write!(f, "{}", "160000"),
        }
    }
}

impl GitIndexMode {
    pub(crate) fn serialize(mode: &GitIndexMode) -> [u8; 4] {
        match &mode {
            GitIndexMode::RegularFile0755 => [0b00000000, 0b00000000, 0b10000001, 0b11101101],
            GitIndexMode::RegularFile0644 => [0b00000000, 0b00000000, 0b10000001, 0b10100100],
            GitIndexMode::SymbolicLink => [0b00000000, 0b00000000, 0b10100000, 0b00000000],
            GitIndexMode::GitLink => [0b00000000, 0b00000000, 0b11100000, 0b00000000],
        }
    }

    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitIndexMode, RustGitError> {
        let bytes: [u8; 4] = bytes.try_into()?;
        match bytes {
            [0b00000000, 0b00000000, 0b10000001, 0b11101101] => Ok(GitIndexMode::RegularFile0755),
            [0b00000000, 0b00000000, 0b10000001, 0b10100100] => Ok(GitIndexMode::RegularFile0644),
            [0b00000000, 0b00000000, 0b10100000, 0b00000000] => Ok(GitIndexMode::SymbolicLink),
            [0b00000000, 0b00000000, 0b11100000, 0b00000000] => Ok(GitIndexMode::GitLink),
            other => Err(RustGitError::new(format!("invalid mode '{other:?}' in index entry")))
        }
    }
}

// Value for each case is important here,
// to maintian consistent ordering of index entries.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum GitIndexStageFlag {
    RegularFileNoConflict = 0,
    Base = 1,
    Ours = 2,
    Theirs = 3,
}

impl Display for GitIndexStageFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitIndexStageFlag::RegularFileNoConflict => write!(f, "{}", "0"),
            GitIndexStageFlag::Base => write!(f, "{}", "1"),
            GitIndexStageFlag::Ours => write!(f, "{}", "2"),
            GitIndexStageFlag::Theirs => write!(f, "{}", "3"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct GitIndexFlags {
    pub assume_valid: bool,
    pub extended: bool,
    pub stage: GitIndexStageFlag,
    pub name_length: u16,
} 

impl GitIndexFlags {
    pub(crate) fn serialize(flags: &GitIndexFlags) -> [u8; 2] {
        let mut first_byte: u8 = 0;
        if flags.assume_valid {
            first_byte |= 0b10000000;
        }
        if flags.extended {
            first_byte |= 0b01000000;
        }

        match flags.stage {
            GitIndexStageFlag::RegularFileNoConflict => (),
            GitIndexStageFlag::Base => first_byte |= 0b00010000,
            GitIndexStageFlag::Ours => first_byte |= 0b00100000,
            GitIndexStageFlag::Theirs => first_byte |= 0b00110000,
        };

        let name_length_bytes = flags.name_length.to_be_bytes();
        first_byte |= name_length_bytes[0] & 0b00001111;

        [first_byte, name_length_bytes[1]]
    }

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

#[derive(Debug)]
pub(crate) struct GitIndexEntry {
    pub last_metadata_update: GitIndexTimestamp,
    pub last_data_update: GitIndexTimestamp,
    pub dev: u32,
    pub ino: u32,
    pub mode: GitIndexMode,
    pub uid: u32,
    pub gid: u32,
    pub file_size: u32,
    pub name: GitObjectId,
    pub flags: GitIndexFlags,

    /// Entry path name (variable length) relative to top level directory
    /// (without leading slash). '/' is used as path separator. The special
    /// path components ".", ".." and ".git" (without quotes) are disallowed.
    /// Trailing slash is also disallowed.
    pub path_name: String,
}

impl GitIndexEntry {
    pub(crate) fn serialize(entry: &GitIndexEntry) -> Vec<u8> {
        let mut fixed: [u8; 62] = [0; 62];
        fixed[0..8].copy_from_slice(&GitIndexTimestamp::serialize(&entry.last_metadata_update));
        fixed[8..16].copy_from_slice(&GitIndexTimestamp::serialize(&entry.last_data_update));
        fixed[16..20].copy_from_slice(&entry.dev.to_be_bytes());
        fixed[20..24].copy_from_slice(&entry.ino.to_be_bytes());
        fixed[24..28].copy_from_slice(&GitIndexMode::serialize(&entry.mode));
        fixed[28..32].copy_from_slice(&entry.uid.to_be_bytes());
        fixed[32..36].copy_from_slice(&entry.gid.to_be_bytes());
        fixed[36..40].copy_from_slice(&entry.file_size.to_be_bytes());
        fixed[40..60].copy_from_slice(&GitObjectId::serialize(&entry.name));
        fixed[60..62].copy_from_slice(&GitIndexFlags::serialize(&entry.flags));

        let path_name_bytes = entry.path_name.as_bytes().to_vec();
        let padding_byte_count = {
            let remainder = (62 + path_name_bytes.len()) % 8;
            if remainder == 0 {
                8
            } else {
                8 - remainder
            }
        };

        let padding_bytes: Vec<u8> = std::iter::repeat(b'\0').take(padding_byte_count).collect();

        let mut bytes = vec![];

        bytes.extend_from_slice(&fixed);
        bytes.extend_from_slice(&path_name_bytes);
        bytes.extend_from_slice(&padding_bytes);

        bytes
    }

    pub(crate) fn deserialize(bytes: &[u8]) -> Result<(GitIndexEntry, usize), RustGitError> {
        let last_metadata_update = GitIndexTimestamp::deserialize(&bytes[0..8])?;
        let last_data_update = GitIndexTimestamp::deserialize(&bytes[8..16])?;
        let dev = as_u32_be(&bytes[16..20].try_into()?);
        let ino = as_u32_be(&bytes[20..24].try_into()?);
        let mode = GitIndexMode::deserialize(&bytes[24..28])?;
        let uid = as_u32_be(&bytes[28..32].try_into()?);
        let gid = as_u32_be(&bytes[32..36].try_into()?);
        let file_size = as_u32_be(&bytes[36..40].try_into()?);
        let name = GitObjectId::deserialize(&bytes[40..60])?;
        let flags = GitIndexFlags::deserialize(&bytes[60..62])?;

        let path_name_bytes = {
            if flags.name_length < 0xFFF {
                Ok(&bytes[62..(62+flags.name_length as usize)])
            } else {
                if let Some(null_index) = &bytes[62..].iter().position(|&b| b == b'\0') {
                    Ok(&bytes[62..62+null_index])
                } else {
                    Err(RustGitError::new("missing null byte for path name"))
                }
            }

        }?;

        let path_name = String::from_utf8(path_name_bytes.to_vec())?;

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
        let mode =
            if metadata.is_symlink() {
                GitIndexMode::SymbolicLink
            } else if metadata.is_dir() {
                GitIndexMode::GitLink
            } else {
                // If file is executable by owner, we set 755, otherwise 644.
                if metadata.permissions().mode() & 0b01000000 != 0 {
                    GitIndexMode::RegularFile0755
                } else {
                    GitIndexMode::RegularFile0644
                }
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
            mode: mode,
            flags: GitIndexFlags {
                assume_valid: false,
                extended: false,
                stage: GitIndexStageFlag::RegularFileNoConflict,
                name_length: path.len() as u16,
            },
            path_name: String::from(path),
        }
    }

    /// Creates a copy of the provided index with a new name.
    pub(crate) fn with_updated_name(self, new_name: &str) -> GitIndexEntry {
        GitIndexEntry { 
            path_name: String::from(new_name), 
            flags: GitIndexFlags { 
                name_length: new_name.len() as u16,
                ..self.flags 
            }, 
            ..self 
        }
    }


}

// Index entries are sorted in ascending order on the name field,
// interpreted as a string of unsigned bytes (i.e. memcmp() order, no
// localization, no special casing of directory separator '/'). Entries
// with the same name are sorted by their stage field.
impl Ord for GitIndexEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.path_name == other.path_name {
            self.flags.stage.cmp(&other.flags.stage)
        } else {
            self.path_name.cmp(&other.path_name)
        }
    }
}

impl PartialOrd for GitIndexEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for GitIndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path_name == other.path_name && self.flags.stage == other.flags.stage
    }
}

impl Eq for GitIndexEntry { }

#[derive(Debug, PartialEq)]
pub(crate) struct GitIndex {
    header: GitIndexHeader,
    entries: Vec<GitIndexEntry>,
}

impl Default for GitIndex {
    fn default() -> Self {
        Self { 
            header: GitIndexHeader {
                version: GitIndexVersion::V2,
                num_entries: 0,
            },
            entries: vec![],
        }
    }
}

impl GitIndex {
    // Reference for Git index binary format: https://git-scm.com/docs/index-format
    pub(crate) fn serialize(index: &GitIndex) -> Vec<u8> {
        let mut bytes = vec![];
        let header_bytes = GitIndexHeader::serialize(&index.header);
        bytes.extend_from_slice(&header_bytes);

        for entry in &index.entries {
            let entry_bytes = GitIndexEntry::serialize(entry);
            bytes.extend_from_slice(&entry_bytes);
        }

        // Skipping extensions for now.

        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        let checksum = hasher.final_oid_fn();
        bytes.extend_from_slice(&GitObjectId::serialize(&checksum));

        bytes
    }

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
        })
    }

    /// Creates a new, empty index.
    pub(crate) fn new() -> GitIndex {
        GitIndex::default()
    }

    /// Loads the index from the provided git directory.
    pub(crate) fn open(git_dir: &Path) -> Result<GitIndex, RustGitError> {
        let index_file_path: PathBuf = git_dir.join(DEFAULT_INDEX_NAME);

        if !index_file_path.exists() {
            return Ok(Self::new());
        }

        let index_file_bytes = fs::read(index_file_path)?;

        let git_index: GitIndex = GitIndex::deserialize(&index_file_bytes)?;

        Ok(git_index)
    }

    /// Writes the index to the provided git directory.
    pub(crate) fn write(&mut self, git_dir: &Path) -> Result<(), RustGitError> {
        let mut index_file = File::create(&git_dir.join(DEFAULT_INDEX_NAME))?;

        let serialized = GitIndex::serialize(&self);

        index_file.write(&serialized)?;

        Ok(())
    }

    /// Inserts the provided entry into the index.
    /// An existing entry with the same path_name and stage will be replaced, otherwise
    /// a new entry will be inserted such that the entry list remains sorted.
    pub(crate) fn add(&mut self, index_entry: GitIndexEntry) {
        let existing_entry = self.entries.binary_search(&index_entry);

        match existing_entry {
            Ok(existing_idx) => self.entries[existing_idx] = index_entry,
            Err(insertion_idx) => {
                self.entries.insert(insertion_idx, index_entry);
                self.header.num_entries += 1;
            }
        }
    }

    /// Returns an iterator over the index entries.
    pub(crate) fn iter_entries(&self) -> impl Iterator<Item=&GitIndexEntry> {
        self.entries.iter()
    }

    /// Returns an iterator over the provided range of index entries.
    pub(crate) fn iter_entries_range(&self, start: usize, end: usize) -> impl Iterator<Item=&GitIndexEntry> {
        self.entries[start..end].iter()
    }

    /// Filters out entries for which the provided predicate returns false.
    /// Returns the path names of the removed entries.
    pub(crate) fn filter_entries(&mut self, mut predicate: impl FnMut(&GitIndexEntry) -> bool) -> Vec<String> {
        let mut removed_paths = Vec::new();
        self.entries.retain(|entry| {
            let result = predicate(entry);
            if !result {
                self.header.num_entries -= 1;
                removed_paths.push(entry.path_name.clone());
            }
            result
        });
        removed_paths
    }

    /// Tries to find an index entry with the provided path.
    pub(crate) fn entry_by_path(&self, path: &Path) -> Option<&GitIndexEntry> {
        self.entries.binary_search_by_key(&path.to_str().unwrap(), |entry| &entry.path_name)
        .map_or(None, |idx| Some(&self.entries[idx]))
    }

    /// Removes the index entry at the provided index.
    /// Assumes that the provided index is valid.
    pub(crate) fn remove_entry_at(&mut self, index: usize) -> GitIndexEntry {
        self.header.num_entries -= 1;
        self.entries.remove(index)
    }

    /// Updates the path name of the index entry with the provided current path name.
    pub(crate) fn rename_entry_by_path(&mut self, current_name: &Path, new_name: &str) {
        if let Some((index, _)) = self.entry_by_path(current_name) {
            let new_entry = self.remove_entry_at(index).with_updated_name(new_name);
            self.add(new_entry);
        }
        // TODO: refresh index entry
        // https://github.com/git/git/blob/master/read-cache.c#L165-L171
    }

    // Returns the range of entries which have the provided path prefix.
    pub(crate) fn entry_range_by_path(&self, prefix: &Path) -> Vec<&GitIndexEntry> {
        self.iter_entries()
        .filter(|entry| PathBuf::from(&entry.path_name).starts_with(prefix))
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::object::GitObjectId;

    use super::{GitIndex, GitIndexEntry, GitIndexFlags, GitIndexMode, GitIndexStageFlag, GitIndexTimestamp};

    fn get_test_index_entry() -> GitIndexEntry { 
        GitIndexEntry {
            last_metadata_update: GitIndexTimestamp {
                seconds: 50,
                nanoseconds: 150,
            },
            last_data_update: GitIndexTimestamp {
                seconds: 100,
                nanoseconds: 200,

            },
            dev: 1,
            ino: 2, 
            mode: GitIndexMode::RegularFile0644,
            uid: 3, 
            gid: 4, 
            file_size: 5, 
            name: GitObjectId::new(String::from("9daeafb9864cf43055ae93beb0afd6c7d144bfa4")),
            flags: GitIndexFlags {
                assume_valid: false,
                extended: false,
                stage: GitIndexStageFlag::RegularFileNoConflict,
                name_length: 8,
            }, 
            path_name: String::from("test.txt")
        }
    }

    #[test]
    fn should_add_entries_in_sorted_order() {
        let mut index = GitIndex::new();
        let entry1 = GitIndexEntry { path_name: String::from("test.txt"), ..get_test_index_entry() };
        let entry2 = GitIndexEntry { path_name: String::from("fest.txt"), ..get_test_index_entry() };
        let entry3 = GitIndexEntry { path_name: String::from("best.txt"), ..get_test_index_entry() };

        index.add(entry1);
        index.add(entry2);
        index.add(entry3);

        assert_eq!(index.entries[0].path_name, "best.txt");
        assert_eq!(index.entries[1].path_name, "fest.txt");
        assert_eq!(index.entries[2].path_name, "test.txt");
    }

    mod serialization {
        use std::iter;

        use super::super::*;

        fn assert_incorrect_bytes_err<T>(expected_bytes: usize, deser_f: impl Fn(&[u8]) -> Result<T, RustGitError>) {
            let too_many_bytes: Vec<u8> = iter::repeat(b'0').take(expected_bytes + 1).collect();
            let too_few_bytes: Vec<u8> = iter::repeat(b'0').take(expected_bytes - 1).collect();
            assert!(deser_f(&too_few_bytes).is_err());
            assert!(deser_f(&too_many_bytes).is_err());
        }

        mod git_index {
            use crate::{error::RustGitError, index::{GitIndex, GitIndexEntry, GitIndexFlags, GitIndexHeader, GitIndexMode, GitIndexStageFlag, GitIndexTimestamp, GitIndexVersion}, object::GitObjectId};

            #[test]
            fn should_roundtrip_empty() {
                let test_index = GitIndex {
                    header: GitIndexHeader {
                        version: GitIndexVersion::V2,
                        num_entries: 0,
                    },
                    entries: vec![],
                };

                let result = GitIndex::deserialize(&GitIndex::serialize(&test_index));
                assert_eq!(result, Ok(test_index));
            }

            #[test]
            fn should_roundtrip() {
                let test_index = GitIndex {
                    header: GitIndexHeader {
                        version: GitIndexVersion::V2,
                        num_entries: 2,
                    },
                    entries: vec![
                        GitIndexEntry { 
                            last_metadata_update: GitIndexTimestamp {
                                seconds: 50,
                                nanoseconds: 150,
                            },
                            last_data_update: GitIndexTimestamp {
                                seconds: 100,
                                nanoseconds: 200,

                            },
                            dev: 1,
                            ino: 2, 
                            mode: GitIndexMode::RegularFile0644,
                            uid: 3, 
                            gid: 4, 
                            file_size: 5, 
                            name: GitObjectId::new(String::from("9daeafb9864cf43055ae93beb0afd6c7d144bfa4")),
                            flags: GitIndexFlags {
                                assume_valid: false,
                                extended: false,
                                stage: GitIndexStageFlag::RegularFileNoConflict,
                                name_length: 8,
                            }, 
                            path_name: String::from("test.txt")
                        },
                        GitIndexEntry { 
                            last_metadata_update: GitIndexTimestamp {
                                seconds: 50,
                                nanoseconds: 150,
                            },
                            last_data_update: GitIndexTimestamp {
                                seconds: 100,
                                nanoseconds: 200,

                            },
                            dev: 1,
                            ino: 2, 
                            mode: GitIndexMode::RegularFile0644,
                            uid: 3, 
                            gid: 4, 
                            file_size: 5, 
                            name: GitObjectId::new(String::from("180cf8328022becee9aaa2577a8f84ea2b9f3827")),
                            flags: GitIndexFlags {
                                assume_valid: false,
                                extended: false,
                                stage: GitIndexStageFlag::RegularFileNoConflict,
                                name_length: 9,
                            }, 
                            path_name: String::from("test2.txt")
                        },

                    ],
                };

                let result = GitIndex::deserialize(&GitIndex::serialize(&test_index));
                assert_eq!(result, Ok(test_index));
            }

            #[test]
            fn should_fail_for_invalid_checksum() {
                let test_index = GitIndex {
                    header: GitIndexHeader {
                        version: GitIndexVersion::V2,
                        num_entries: 0,
                    },
                    entries: vec![],
                };

                let mut serialized_bytes = GitIndex::serialize(&test_index);
                let num_bytes = serialized_bytes.len();
                let real_checksum = hex::encode(&serialized_bytes[num_bytes-20..]);
                serialized_bytes[num_bytes-1] = 0;
                let invalid_checksum = hex::encode(&serialized_bytes[num_bytes-20..]);

                let result = GitIndex::deserialize(&serialized_bytes);
                
                assert_eq!(result, Err(RustGitError::new(format!("Index checksum {invalid_checksum} doesn't match computed hash {real_checksum}"))));
            }
        }

        mod git_index_entry {
            use std::iter;

            use crate::index::{tests::get_test_index_entry, GitIndexEntry, GitIndexFlags};

            #[test]
            fn should_roundtrip() {
                let git_index_entry = get_test_index_entry();
                let result = GitIndexEntry::deserialize(&GitIndexEntry::serialize(&git_index_entry));
                assert_eq!(result, Ok((git_index_entry, 72)));
            }

            #[test]
            fn should_roundtrip_long_name() {
                let test_index_entry = get_test_index_entry();
                let git_index_entry = GitIndexEntry { 
                    flags: GitIndexFlags { name_length: 0xFFF, ..test_index_entry.flags },
                    path_name: iter::repeat('A').take(0xFFF1).collect(),
                    ..test_index_entry
                };
                let result = GitIndexEntry::deserialize(&GitIndexEntry::serialize(&git_index_entry));
                assert_eq!(result, Ok((git_index_entry, 65584)));
            }
        }

        mod git_index_flags {
            use super::{assert_incorrect_bytes_err, GitIndexFlags};

            #[test]
            fn should_roundtrip() {
                let git_index_flags = GitIndexFlags {
                    assume_valid: false,
                    extended: false,
                    stage: super::GitIndexStageFlag::RegularFileNoConflict,
                    name_length: 5,
                };

                let result = GitIndexFlags::deserialize(&GitIndexFlags::serialize(&git_index_flags));
                assert_eq!(result, Ok(git_index_flags));
            }

            #[test]
            fn should_fail_for_incorrect_number_of_bytes() {
                assert_incorrect_bytes_err(2, GitIndexFlags::deserialize)
            }
        }

        mod git_index_mode {
            use crate::error::RustGitError;

            use super::{assert_incorrect_bytes_err, GitIndexMode};

            #[test]
            fn should_roundtrip() {
                assert_eq!(GitIndexMode::deserialize(&GitIndexMode::serialize(&GitIndexMode::RegularFile0755)), Ok(GitIndexMode::RegularFile0755));
                assert_eq!(GitIndexMode::deserialize(&GitIndexMode::serialize(&GitIndexMode::RegularFile0644)), Ok(GitIndexMode::RegularFile0644));
                assert_eq!(GitIndexMode::deserialize(&GitIndexMode::serialize(&GitIndexMode::SymbolicLink)), Ok(GitIndexMode::SymbolicLink));
                assert_eq!(GitIndexMode::deserialize(&GitIndexMode::serialize(&GitIndexMode::GitLink)), Ok(GitIndexMode::GitLink));
            }

            #[test]
            fn should_fail_for_incorrect_number_of_bytes() {
                assert_incorrect_bytes_err(4, GitIndexMode::deserialize);
            }

            #[test]
            fn should_fail_for_invalid_mode() {
                let bytes: &[u8] = &[0, 0, 0, 0];
                let result = GitIndexMode::deserialize(&bytes);
                assert_eq!(result, Err(RustGitError::new("invalid mode '[0, 0, 0, 0]' in index entry")));
            }

        }

        mod git_index_timestamp {
            use super::{assert_incorrect_bytes_err, GitIndexTimestamp};

            #[test]
            fn should_roundtrip() {
                let git_index_timestamp = GitIndexTimestamp {
                    seconds: 50,
                    nanoseconds: 100,
                };

                let result = GitIndexTimestamp::deserialize(&GitIndexTimestamp::serialize(&git_index_timestamp));
                assert_eq!(result, Ok(git_index_timestamp));
            }

            #[test]
            fn should_fail_for_incorrect_number_of_bytes() {
                assert_incorrect_bytes_err(8, GitIndexTimestamp::deserialize);
            }
        }

        mod git_index_header {
            use crate::error::RustGitError;

            use super::{assert_incorrect_bytes_err, GitIndexVersion, GitIndexHeader};

            #[test]
            fn should_roundtrip() {
                let git_index_header = GitIndexHeader {
                    version: GitIndexVersion::V2,
                    num_entries: 2,
                };

                let result = GitIndexHeader::deserialize(&GitIndexHeader::serialize(&git_index_header));
                assert_eq!(result, Ok(git_index_header));
            }

            #[test]
            fn should_fail_for_incorrect_number_of_bytes() {
                assert_incorrect_bytes_err(12, GitIndexHeader::deserialize);
            }

            #[test]
            fn should_fail_for_missing_signature() {
                let bytes: &[u8] = b"LIRC00000000"; // should be DIRC
                let result = GitIndexHeader::deserialize(&bytes);
                assert_eq!(result, Err(RustGitError::new("missing header signature in index file")));
            }
        }

        mod git_index_version {
            use crate::error::RustGitError;

            use super::{assert_incorrect_bytes_err, GitIndexVersion};

            #[test]
            fn should_roundtrip() {
                let git_index_version = GitIndexVersion::V2;

                let result = GitIndexVersion::deserialize(&GitIndexVersion::serialize(&git_index_version));
                assert_eq!(result, Ok(git_index_version));
            }

            #[test]
            fn should_fail_for_incorrect_number_of_bytes() {
                assert_incorrect_bytes_err(4, GitIndexVersion::deserialize);
            }

            #[test]
            fn should_fail_for_invalid_version() {
                let result = GitIndexVersion::deserialize(&(3 as u32).to_be_bytes());
                assert_eq!(result, Err(RustGitError::new("unsupported index version 3")));
            }
        }
    }
}