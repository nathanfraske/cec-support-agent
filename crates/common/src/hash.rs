/// Stable, order-independent FNV-1a content hash over string keys, rendered as
/// a fixed-width hex string. Non-cryptographic; it carries no identity data and
/// pulls in no dependencies. Shared by [`FaultSignature`](crate::FaultSignature)
/// fingerprints and [`ConfigClass`](crate::ConfigClass) derived hashes.
pub(crate) fn fingerprint_of(keys: &[&str]) -> String {
    let mut keys: Vec<&str> = keys.to_vec();
    keys.sort_unstable();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for key in keys {
        for byte in key.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        // Separator so ["ab","c"] and ["a","bc"] do not collide.
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}
