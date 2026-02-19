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
    const INTERNAL_VK_LEN: usize = 4 * 8 + 27 * 64; // 1760 bytes
    const BB_VK_LEN: usize = 3 * 32 + 28 * 64; // 1888 bytes
    const BB_Q_NNF_POINT_INDEX: usize = 11; // Present in bb vk, absent in internal verifier vk

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

    fn hex_value(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(10 + (b - b'a')),
            b'A'..=b'F' => Some(10 + (b - b'A')),
            _ => None,
        }
    }

    fn hex_str_to_be32(s: &str) -> Option<[u8; 32]> {
        let hex = s.trim_start_matches("0x").as_bytes();
        let mut out = [0u8; 32];
        let mut oi = 32usize;
        let mut i = hex.len();
        while i > 0 && oi > 0 {
            let low = Self::hex_value(hex[i - 1])?;
            i -= 1;
            let high = if i > 0 {
                let v = Self::hex_value(hex[i - 1])?;
                i -= 1;
                v
            } else {
                0
            };
            oi -= 1;
            out[oi] = (high << 4) | low;
        }
        Some(out)
    }

    // Matches verifier crate's limb composition:
    // out[..15] = hi[17..], out[15..] = lo[15..]
    fn combine_limbs(lo: &[u8; 32], hi: &[u8; 32]) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[..15].copy_from_slice(&hi[17..]);
        out[15..].copy_from_slice(&lo[15..]);
        out
    }

    fn parse_json_array_of_strings(s: &str) -> Option<StdVec<StdVec<u8>>> {
        let bytes = s.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'[' {
            return None;
        }
        i += 1;

        let mut out: StdVec<StdVec<u8>> = StdVec::new();
        loop {
            while i < bytes.len() && ((bytes[i] as char).is_whitespace() || bytes[i] == b',') {
                i += 1;
            }
            if i >= bytes.len() {
                return None;
            }
            if bytes[i] == b']' {
                break;
            }
            if bytes[i] != b'"' {
                return None;
            }
            i += 1;
            let mut token: StdVec<u8> = StdVec::new();
            while i < bytes.len() {
                let c = bytes[i];
                i += 1;
                if c == b'"' {
                    break;
                }
                if c == b'\\' {
                    if i >= bytes.len() {
                        return None;
                    }
                    token.push(bytes[i]);
                    i += 1;
                } else {
                    token.push(c);
                }
            }
            out.push(token);
        }
        Some(out)
    }

    fn try_build_vk_bytes_from_fields(
        fields: &[StdVec<u8>],
        points_start: usize,
        total_points: usize,
        skip_point_index: Option<usize>,
    ) -> Option<StdVec<u8>> {
        if fields.len() < points_start + (total_points * 4) {
            return None;
        }

        let f0 = Self::hex_str_to_be32(core::str::from_utf8(&fields[0]).ok()?)?;
        let f1 = Self::hex_str_to_be32(core::str::from_utf8(&fields[1]).ok()?)?;
        let f2 = Self::hex_str_to_be32(core::str::from_utf8(&fields[2]).ok()?)?;
        let h0 = Self::field32_to_u64_be(&f0)?;
        let public_inputs_size = Self::field32_to_u64_be(&f1)?;
        let pub_inputs_offset = Self::field32_to_u64_be(&f2)?;
        let (circuit_size, log_n) = Self::decode_circuit_size_and_log_n(h0)?;

        let mut out = StdVec::with_capacity(Self::INTERNAL_VK_LEN);
        Self::push_u64_be(&mut out, circuit_size);
        Self::push_u64_be(&mut out, log_n);
        Self::push_u64_be(&mut out, public_inputs_size);
        Self::push_u64_be(&mut out, pub_inputs_offset);

        let mut idx = points_start;
        for point_idx in 0..total_points {
            let lx = Self::hex_str_to_be32(core::str::from_utf8(&fields[idx]).ok()?)?;
            let hx = Self::hex_str_to_be32(core::str::from_utf8(&fields[idx + 1]).ok()?)?;
            let ly = Self::hex_str_to_be32(core::str::from_utf8(&fields[idx + 2]).ok()?)?;
            let hy = Self::hex_str_to_be32(core::str::from_utf8(&fields[idx + 3]).ok()?)?;
            idx += 4;

            if Some(point_idx) == skip_point_index {
                continue;
            }

            let x = Self::combine_limbs(&lx, &hx);
            let y = Self::combine_limbs(&ly, &hy);
            out.extend_from_slice(&x);
            out.extend_from_slice(&y);
        }

        if out.len() != Self::INTERNAL_VK_LEN {
            return None;
        }
        Some(out)
    }

    fn vk_from_fields_json_str(vk_json: &str) -> Option<StdVec<u8>> {
        let fields = Self::parse_json_array_of_strings(vk_json)?;
        if fields.len() < (3 + (27 * 4)) {
            return None;
        }

        // bb 0.87 vk_fields often has:
        // [header0, header1, header2, marker, 27 points * 4 limbs]
        if let Some(vk) = Self::try_build_vk_bytes_from_fields(&fields, 4, 27, None) {
            return Some(vk);
        }
        // Legacy layout:
        // [header0, header1, header2, 28 points * 4 limbs], where one point is q_nnf.
        Self::try_build_vk_bytes_from_fields(&fields, 3, 28, Some(Self::BB_Q_NNF_POINT_INDEX))
    }

    fn push_u64_be(out: &mut StdVec<u8>, v: u64) {
        out.extend_from_slice(&v.to_be_bytes());
    }

    // bb vk headers are encoded as 32-byte field elements; interpret low 8 bytes as u64.
    fn field32_to_u64_be(field: &[u8]) -> Option<u64> {
        if field.len() != 32 {
            return None;
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&field[24..32]);
        Some(u64::from_be_bytes(buf))
    }

    fn bytes_from_slice(env: &Env, bytes: &[u8]) -> Bytes {
        let mut out = Bytes::new(env);
        for b in bytes {
            out.push_back(*b);
        }
        out
    }

    // Header word0 can be either circuit_size (power-of-two) or log_n.
    fn decode_circuit_size_and_log_n(h0: u64) -> Option<(u64, u64)> {
        if h0 == 0 {
            return None;
        }
        if (h0 & (h0 - 1)) == 0 {
            let mut lg = 0u64;
            let mut n = h0;
            while n > 1 {
                n >>= 1;
                lg += 1;
            }
            Some((h0, lg))
        } else {
            let circuit_size = 1u64.checked_shl(h0 as u32)?;
            Some((circuit_size, h0))
        }
    }

    // Accept either:
    // - internal verifier vk bytes format (1760 bytes), or
    // - bb vk bytes format (1888 bytes; 3 field headers + 28 points).
    fn normalize_vk_bytes(raw: &Bytes) -> Option<StdVec<u8>> {
        let raw_vec = raw.to_alloc_vec();
        if raw_vec.len() == Self::INTERNAL_VK_LEN {
            return Some(raw_vec);
        }
        if raw_vec.len() != Self::BB_VK_LEN {
            return None;
        }

        let h0 = Self::field32_to_u64_be(&raw_vec[0..32])?;
        let public_inputs_size = Self::field32_to_u64_be(&raw_vec[32..64])?;
        let pub_inputs_offset = Self::field32_to_u64_be(&raw_vec[64..96])?;
        let (circuit_size, log_n) = Self::decode_circuit_size_and_log_n(h0)?;

        let mut out = StdVec::with_capacity(Self::INTERNAL_VK_LEN);
        Self::push_u64_be(&mut out, circuit_size);
        Self::push_u64_be(&mut out, log_n);
        Self::push_u64_be(&mut out, public_inputs_size);
        Self::push_u64_be(&mut out, pub_inputs_offset);

        let points_bytes = &raw_vec[96..];
        for point_idx in 0..28usize {
            if point_idx == Self::BB_Q_NNF_POINT_INDEX {
                continue;
            }
            let start = point_idx * 64;
            let end = start + 64;
            out.extend_from_slice(&points_bytes[start..end]);
        }

        if out.len() != Self::INTERNAL_VK_LEN {
            return None;
        }
        Some(out)
    }

    fn split_inputs_and_proof_bytes(packed: &[u8]) -> Option<(StdVec<u8>, StdVec<u8>)> {
        if packed.len() < 4 {
            return None;
        }
        let rest = &packed[4..];

        for &proof_fields in &[456usize, 440usize, 234usize] {
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

    pub fn verify_proof(env: Env, vk_bytes: Bytes, proof_blob: Bytes) -> Result<BytesN<32>, Error> {
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

        let verifier = UltraHonkVerifier::new(&env, &vk_bytes).map_err(|_| Error::VkParseError)?;
        verifier
            .verify(&proof_bytes, &public_inputs)
            .map_err(|_| Error::VerificationFailed)?;

        let proof_hash = Self::keccak32(&proof_blob_vec);
        let proof_id = BytesN::from_array(&env, &proof_hash);
        env.storage().instance().set(&proof_id, &true);
        Ok(proof_id)
    }

    fn store_vk_bytes(env: &Env, vk_bytes: Bytes) -> Result<BytesN<32>, Error> {
        let normalized = Self::normalize_vk_bytes(&vk_bytes).ok_or(Error::VkParseError)?;
        let normalized_bytes = Self::bytes_from_slice(env, &normalized);
        env.storage().instance().set(&Self::key_vk(), &normalized_bytes);
        let vk_vec = normalized_bytes.to_alloc_vec();
        let hash_arr = Self::keccak32(&vk_vec);
        let hash_bn = BytesN::from_array(env, &hash_arr);
        env.storage().instance().set(&Self::key_vk_hash(), &hash_bn);
        Ok(hash_bn)
    }

    pub fn set_vk_bytes(env: Env, vk: Bytes) -> Result<BytesN<32>, Error> {
        if vk.len() == 0 {
            return Err(Error::VkParseError);
        }
        Self::store_vk_bytes(&env, vk)
    }

    pub fn set_vk(env: Env, vk_json: String) -> Result<BytesN<32>, Error> {
        let vk_vec = vk_json.to_bytes().to_alloc_vec();
        let vk_str = core::str::from_utf8(&vk_vec).map_err(|_| Error::VkParseError)?;
        let normalized = Self::vk_from_fields_json_str(vk_str).ok_or(Error::VkParseError)?;
        let vk_bytes = Self::bytes_from_slice(&env, &normalized);
        UltraHonkVerifier::new(&env, &vk_bytes).map_err(|_| Error::VkParseError)?;
        Self::store_vk_bytes(&env, vk_bytes)
    }

    pub fn verify_proof_with_stored_vk(env: Env, proof_blob: Bytes) -> Result<BytesN<32>, Error> {
        let vk_bytes: Bytes = env
            .storage()
            .instance()
            .get(&Self::key_vk())
            .ok_or(Error::VkNotSet)?;
        Self::verify_proof(env, vk_bytes, proof_blob)
    }

    pub fn is_verified(env: Env, proof_id: BytesN<32>) -> bool {
        env.storage().instance().get(&proof_id).unwrap_or(false)
    }
}
