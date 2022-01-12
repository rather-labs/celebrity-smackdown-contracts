use crate::error::Error;
use crate::helper::{parse_dyn_vec_len, DYN_MIN_LEN};
use alloc::vec::Vec;
use core::result::Result;

const FIXED_LEN: usize = 1;
pub const NFT_DATA_MIN_LEN: usize = 3;
pub const NFT_TYPE_ARGS_LEN: usize = 56;

/// NFT cell data structure
/// This structure contains the following information:
/// 1) version: u8
/// 2) data: <size: u16> + <vartext>
#[derive(Debug, Clone)]
pub struct Nft {
  pub version: u8,
  pub data: Vec<u8>,
}

impl Nft {
  pub fn from_data(raw_data: &[u8]) -> Result<Self, Error> {
    if raw_data.len() < NFT_DATA_MIN_LEN {
      return Err(Error::NFTDataInvalid);
    }

    let version: u8 = raw_data[0];
    if version != 0 {
      return Err(Error::VersionInvalid);
    }

    let data_len = parse_dyn_vec_len(&raw_data[FIXED_LEN..(FIXED_LEN + DYN_MIN_LEN)]);
    let data = raw_data[FIXED_LEN..(FIXED_LEN + data_len)].to_vec();

    Ok(Nft { version, data })
  }
}
