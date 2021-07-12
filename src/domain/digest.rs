use sha2::Digest;
use sha2::Sha256;

pub type HashDigest = [u8; 32];

pub trait AsBytes {
  fn as_bytes(&self) -> Vec<u8>;
}

pub trait Hashable: AsBytes {
  fn hash(&self) -> HashDigest {
    let digest: Vec<u8> = Sha256::digest(&self.as_bytes()).into_iter().collect();
    let mut hash_digest: HashDigest = [0; 32];
    hash_digest[..32].clone_from_slice(&digest[..32]);
    hash_digest
  }
}
