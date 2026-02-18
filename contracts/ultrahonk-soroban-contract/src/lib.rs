#![no_std]
extern crate alloc;

use alloc::vec::Vec as StdVec;
use sha3::{Digest, Keccak256};
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Bytes, BytesN, Env, String, Symbol,
};
use ultrahonk_soroban_verifier::UltraHonkVerifier;

#[contract]
pub struct UltraHonkVerifierContract;

#[contracterror]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    VkParseError = 1,
    ProofParseError = 2,
    VerificationFailed = 3,
    VkNotSet = 4,
}

#[contractimpl]
impl UltraHonkVerifierContract {
    fn key_vk() -> Symbol {
        symbol_short!("vk")
    }

    fn key_vk_hash() -> Symbol {
        symbol_short!("vk_hash")
    }

    fn keccak32(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    }

    fn split_inputs_and_proof_bytes(packed: &[u8]) -> Option<(StdVec<u8>, StdVec<u8>)> {
        if packed.len() < 4 {
            return None;
        }
        let rest = &packed[4..];

        for &proof_fields in &[456usize, 440usize] {
            let proof_len = proof_fields * 32;
            if rest.len() < proof_len {
                continue;
            }
            let public_inputs_len = rest.len() - proof_len;
            if public_inputs_len % 32 != 0 {
                continue;
            }
            let public_inputs = rest[..public_inputs_len].to_vec();
            let proof = rest[public_inputs_len..].to_vec();
            return Some((public_inputs, proof));
        }

        None
    }

    pub fn verify_proof(env: Env, vk_json: Bytes, proof_blob: Bytes) -> Result<BytesN<32>, Error> {
        let proof_blob_vec = proof_blob.to_alloc_vec();
        let (public_inputs_vec, proof_vec) =
            Self::split_inputs_and_proof_bytes(&proof_blob_vec).ok_or(Error::ProofParseError)?;

        let mut public_inputs = Bytes::new(&env);
        for b in &public_inputs_vec {
            public_inputs.push_back(*b);
        }

        let mut proof_bytes = Bytes::new(&env);
        for b in &proof_vec {
            proof_bytes.push_back(*b);
        }

        let verifier = UltraHonkVerifier::new(&env, &vk_json).map_err(|_| Error::VkParseError)?;
        verifier
            .verify(&proof_bytes, &public_inputs)
            .map_err(|_| Error::VerificationFailed)?;

        let proof_hash = Self::keccak32(&proof_blob_vec);
        let proof_id = BytesN::from_array(&env, &proof_hash);
        env.storage().instance().set(&proof_id, &true);
        Ok(proof_id)
    }

    pub fn set_vk(env: Env, vk_json: String) -> Result<BytesN<32>, Error> {
        let vk_bytes = vk_json.to_bytes();
        env.storage().instance().set(&Self::key_vk(), &vk_bytes);
        let vk_vec = vk_bytes.to_alloc_vec();
        let hash_arr = Self::keccak32(&vk_vec);
        let hash_bn = BytesN::from_array(&env, &hash_arr);
        env.storage().instance().set(&Self::key_vk_hash(), &hash_bn);
        Ok(hash_bn)
    }

    pub fn verify_proof_with_stored_vk(env: Env, proof_blob: Bytes) -> Result<BytesN<32>, Error> {
        let vk_json: Bytes = env
            .storage()
            .instance()
            .get(&Self::key_vk())
            .ok_or(Error::VkNotSet)?;
        Self::verify_proof(env, vk_json, proof_blob)
    }

    pub fn is_verified(env: Env, proof_id: BytesN<32>) -> bool {
        env.storage().instance().get(&proof_id).unwrap_or(false)
    }
}
