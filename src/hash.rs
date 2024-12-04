use crate::{commands::porcelain::init::cli::HashAlgorithm, object::id::GitObjectId};
use sha1::{Digest, Sha1};

pub(crate) trait Hasher {
    fn name(&self) -> HashAlgorithm;
    fn update_fn(&mut self, content: &str);
    fn final_oid_fn(&mut self) -> GitObjectId;
}

impl Hasher for Sha1 {
    fn name(&self) -> HashAlgorithm {
        HashAlgorithm::Sha1
    }

    fn update_fn(&mut self, content: &str) {
        self.update(content.as_bytes())
    }

    fn final_oid_fn(&mut self) -> GitObjectId {
        let result = self.finalize_reset();
        let s = hex::encode(result[..].to_vec());
        GitObjectId::new(s)
    }
}

pub(crate) fn get_hasher(hash_algo: HashAlgorithm) -> Box<dyn Hasher> {
    match hash_algo {
        HashAlgorithm::Sha1 => Box::new(Sha1::new()),
        HashAlgorithm::Sha256 => todo!(),
    }
}
