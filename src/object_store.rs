use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::error::RustGitError;
use crate::object::{GitObject, GitObjectContents, GitObjectId};

use flate2::read::ZlibDecoder;
use flate2::{write::ZlibEncoder, Compression};
use std::fs::File;
use std::io::{Read, Write};

const OBJECTS_FOLDER: &str = "objects";

pub(crate) struct GitObjectStore {
    /// Path to object store folder.
    obj_dir: PathBuf,
}

impl GitObjectStore {
    pub(crate) fn new(git_dir: &Path) -> GitObjectStore {
        GitObjectStore {
            obj_dir: git_dir.join(OBJECTS_FOLDER),
        }
    }

    pub(crate) fn loose_object_path(&self, obj_id: &GitObjectId) -> (PathBuf, PathBuf) {
        // C Git additional logic omitted:
        // https://github.com/git/git/blob/11c821f2f2a31e70fb5cc449f9a29401c333aad2/object-file.c#L436-L445

        let (folder_name, file_name) = obj_id.folder_and_file_name();

        let obj_folder = self.obj_dir.join(folder_name);

        (obj_folder, Path::new(file_name).to_path_buf())
    }

    pub(crate) fn write_object<T>(&self, obj: T) -> Result<GitObjectId, RustGitError>
    where
        T: TryInto<GitObject, Error = RustGitError>,
    {
        let obj: GitObject = obj.try_into()?;
        self.write_raw_object(&obj)?;
        return Ok(obj.id);
    }

    pub(crate) fn write_raw_object(&self, obj: &GitObject) -> Result<(), RustGitError> {
        // C Git has much more additional logic here, we just implement the core indexing logic to keep things simple:
        // - C Git implementation: https://github.com/git/git/blob/master/object-file.c#L2448
        // - C Git core indexing function: https://github.com/git/git/blob/master/object-file.c#L2312

        // Omitted blob conversion: https://github.com/git/git/blob/master/object-file.c#L2312
        // Omitted hash format check: https://github.com/git/git/blob/master/object-file.c#L2335-L2343

        let (obj_folder, obj_file_name) = self.loose_object_path(&obj.id);

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(obj.content.to_string().as_bytes())?;
        let compressed_bytes = encoder.finish()?;

        create_dir_all(&obj_folder)?;
        let obj_file_path = obj_folder.join(obj_file_name);
        let mut object_file = File::create(obj_file_path)?;
        object_file.write_all(&compressed_bytes)?;

        Ok(())
    }

    pub(crate) fn read_object(
        &self,
        obj_id: &GitObjectId,
    ) -> Result<Option<GitObjectContents>, RustGitError> {
        let (obj_folder, obj_file_name) = self.loose_object_path(&obj_id);
        let obj_file_path = obj_folder.join(obj_file_name);

        if !obj_file_path.exists() {
            return Ok(None);
        }

        let object_file = File::open(obj_file_path)?;

        let mut decoder = ZlibDecoder::new(object_file);
        let mut decoded = String::new();
        decoder.read_to_string(&mut decoded).unwrap();
        let obj = decoded.parse::<GitObjectContents>()?;

        Ok(Some(obj))
    }

    /// Returns true if the provided object id exists in the repo.
    pub(crate) fn is_valid_object_id(&self, obj_id: &GitObjectId) -> bool {
        let (obj_folder, obj_file_name) = self.loose_object_path(&obj_id);
        let obj_file_path = obj_folder.join(obj_file_name);

        return obj_file_path.exists();
    }
}
