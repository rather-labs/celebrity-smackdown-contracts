use crate::error::Error;
use crate::helper::{parse_dyn_vec_len, DYN_MIN_LEN};
use core::result::Result;
use alloc::vec::Vec;

const FIXED_LEN: usize = 0;
// FIXED_LEN + DYN_MIN_LEN
const METADATA_DATA_MIN_LEN: usize = 4;
pub const METADATA_TYPE_ARGS_LEN: usize = 20;

/// Metadata cell data structure
/// This structure contains the following information:
/// 1) name: <size: u16> + <content>
/// 2) data: <size: u32> + <content>
#[derive(Debug, Clone)]
pub struct Metadata {
  pub name: Vec<u8>,
  pub data: Vec<u8>,
}

impl Metadata {
  pub fn from_data(data_raw: &[u8]) -> Result<Self, Error> {
    if data_raw.len() < METADATA_DATA_MIN_LEN {
      return Err(Error::MetadataDataInvalid);
    }

    let name_len = parse_dyn_vec_len(&data_raw[FIXED_LEN..(FIXED_LEN + DYN_MIN_LEN)]);
    // DYN_MIN_LEN: the min length of data
    if data_raw.len() < FIXED_LEN + name_len + DYN_MIN_LEN {
      return Err(Error::MetadataDataInvalid);
    }
    let name = data_raw[FIXED_LEN..(FIXED_LEN + name_len)].to_vec();

    let data_index = FIXED_LEN + name_len;
    let data_len =
      parse_dyn_vec_len(&data_raw[data_index..(data_index + DYN_MIN_LEN)]);
    let data = data_raw[data_index..(data_index + data_len)].to_vec();

    Ok(Metadata {
      name,
      data,
    })
  }
  pub fn immutable_equal(&self, other: &Metadata) -> bool {
    self.name == other.name
  }
}
