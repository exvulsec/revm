use crate::{bytes, Address, Bytes};

/// EIP-7702 Version Magic in u16 form.
pub const EIP7702_MAGIC: u16 = 0xEF01;

/// EIP-7702 magic number in array form.
pub static EIP7702_MAGIC_BYTES: Bytes = bytes!("ef01");

/// EIP-7702 first version of bytecode.
pub const EIP7702_VERSION: u8 = 0;

/// Bytecode of delegated account, specified in EIP-7702
///
/// Format of EIP-7702 bytecode consist of:
/// 0xEF00 (MAGIC) + 0x00 (VERSION) + 20 bytes of address.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eip7702Bytecode {
    pub delegated_address: Address,
    pub version: u8,
    pub raw: Bytes,
}

impl Eip7702Bytecode {
    /// Creates a new EIP-7702 bytecode or returns None if the raw bytecode is invalid.
    #[inline]
    pub fn new_raw(raw: Bytes) -> Option<Self> {
        if raw.len() != 23 {
            return None;
        }
        if !raw.starts_with(&EIP7702_MAGIC_BYTES) {
            return None;
        }

        // Only supported version is version 0.
        if raw[2] != EIP7702_VERSION {
            return None;
        }

        Some(Self {
            delegated_address: Address::new(raw[3..].try_into().unwrap()),
            version: EIP7702_VERSION,
            raw,
        })
    }

    /// Creates a new EIP-7702 bytecode with the given address.
    pub fn new(address: Address) -> Self {
        let mut raw = EIP7702_MAGIC_BYTES.to_vec();
        raw.push(EIP7702_VERSION);
        raw.extend(&address);
        Self {
            delegated_address: address,
            version: EIP7702_VERSION,
            raw: raw.into(),
        }
    }

    /// Return the raw bytecode with version MAGIC number.
    #[inline]
    pub fn raw(&self) -> &Bytes {
        &self.raw
    }

    /// Return the address of the delegated contract.
    #[inline]
    pub fn address(&self) -> Address {
        self.delegated_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_decode() {
        let raw = bytes!("ef01deadbeef");
        assert_eq!(Eip7702Bytecode::new_raw(raw), None);
        let raw = bytes!("ef0100deadbeef00000000000000000000000000000000");
        let address = raw[3..].try_into().unwrap();
        assert_eq!(
            Eip7702Bytecode::new_raw(raw.clone()),
            Some(Eip7702Bytecode {
                delegated_address: address,
                version: 0,
                raw,
            })
        );
    }

    #[test]
    fn create_eip7702_bytecode_from_address() {
        let address = Address::new([0x01; 20]);
        let bytecode = Eip7702Bytecode::new(address);
        assert_eq!(bytecode.delegated_address, address);
        assert_eq!(
            bytecode.raw,
            bytes!("ef01000101010101010101010101010101010101010101")
        );
    }
}
