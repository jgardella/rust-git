use crate::{index::ObjectId, init::cli::HashAlgorithm};
use sha1::{Sha1, Digest};

pub(crate) trait Hasher {
    fn name(&self) -> HashAlgorithm;
    fn update_fn(&mut self, content: String);
    fn final_oid_fn(&mut self) -> ObjectId;
}

impl Hasher for Sha1 {
    fn name(&self) -> HashAlgorithm {
        HashAlgorithm::Sha1
    }

    fn update_fn(&mut self, content: String) {
        self.update(content.as_bytes())
    }

    fn final_oid_fn(&mut self) -> ObjectId  {
        let result = self.finalize_reset();
        let s = hex::encode(result[..].to_vec());
        ObjectId::new(s)
    }
}

pub(crate) fn get_hasher(hash_algo: HashAlgorithm) -> Box<dyn Hasher> {
    match hash_algo {
        HashAlgorithm::Sha1 => Box::new(Sha1::new()),
        HashAlgorithm::Sha256 => todo!(),
    }
}
