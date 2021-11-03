use crate::class::CLASS_TYPE_ARGS_LEN;
use crate::error::Error;
use crate::issuer::ISSUER_TYPE_ARGS_LEN;
use alloc::vec::Vec;
use ckb_std::{
  ckb_constants::Source,
  ckb_types::{bytes::Bytes, packed::*, prelude::*},
  high_level::{
    load_cell_data, load_cell_lock, load_cell_type, load_cell_type_hash, load_witness_args,
    QueryIter, load_cell_capacity,
  },
};
use core::result::Result;

const ID_LEN: usize = 4;
pub const DYN_MIN_LEN: usize = 2; // the length of dynamic data size(u16)

const TYPE: u8 = 1;
const CLASS_TYPE_CODE_HASH: [u8; 32] = [
   78,   8,  42, 115, 174, 100,  11,  46,
  187, 192, 210, 204, 177, 131, 115, 254,
  240, 240, 143,  58, 171, 103, 241,  20,
   16, 190, 248, 188,  64, 150,  75, 124
];

const PAYMENT_TYPE_CODE_HASH: [u8; 32] = [
  155, 215, 224, 111,  62, 207, 75,
  224, 242, 252, 210,  24, 139, 35,
  241, 185, 252, 200, 142,  93, 75,
  101, 168,  99, 123,  23, 114, 59,
  189, 163, 204, 232
];

const PAYMENT_TYPE_ARGS: [u8; 20] = [
   39, 148,  43, 226, 141,  38,
   48, 178, 237, 234, 223, 164,
  245, 142, 220, 147, 231, 104,
  127, 126
];

pub enum Action {
  Create,
  Update,
  Destroy,
}

fn parse_type_args_id(type_script: Script, slice_start: usize) -> Option<u32> {
  let type_args: Bytes = type_script.args().unpack();
  let id_slice = &type_args[slice_start..];
  if id_slice.len() != ID_LEN {
    return None;
  }
  let mut ids = [0u8; ID_LEN];
  ids.copy_from_slice(&id_slice[..]);
  Some(u32::from_be_bytes(ids))
}

fn parse_type_opt(type_opt: &Option<Script>, predicate: &dyn Fn(&Script) -> bool) -> bool {
  match type_opt {
    Some(type_) => predicate(type_),
    None => false,
  }
}

pub fn load_class_type(nft_args: &Bytes) -> Script {
  Script::new_builder()
    .code_hash(CLASS_TYPE_CODE_HASH.pack())
    .args(nft_args[0..CLASS_TYPE_ARGS_LEN].pack())
    .hash_type(Byte::new(TYPE))
    .build()
}

pub fn count_cells_by_type(source: Source, predicate: &dyn Fn(&Script) -> bool) -> usize {
  QueryIter::new(load_cell_type, source)
    .filter(|type_opt| parse_type_opt(&type_opt, predicate))
    .count()
}

pub fn count_cells_by_type_hash(source: Source, predicate: &dyn Fn(&[u8]) -> bool) -> usize {
  QueryIter::new(load_cell_type_hash, source)
    .filter(|type_hash_opt| type_hash_opt.map_or(false, |type_hash| predicate(&type_hash)))
    .count()
}

pub fn load_output_index_by_type(type_script: &Script) -> Option<usize> {
  QueryIter::new(load_cell_type, Source::Output)
    .position(|type_opt| type_opt.map_or(false, |type_| type_.as_slice() == type_script.as_slice()))
}

pub fn load_cell_data_by_type(
  source: Source,
  predicate: &dyn Fn(&Script) -> bool,
) -> Option<Vec<u8>> {
  QueryIter::new(load_cell_type, source)
    .position(|type_opt| type_opt.map_or(false, |type_| predicate(&type_)))
    .map(|index| load_cell_data(index, source).map_or_else(|_| Vec::new(), |data| data))
}

pub fn load_payment_cell_capacity() -> Result<u64, Error> {
  let lock_script: Script = Script::new_builder()
    .code_hash(PAYMENT_TYPE_CODE_HASH.pack())
    .args(PAYMENT_TYPE_ARGS.pack())
    .hash_type(Byte::new(TYPE))
    .build();

  QueryIter::new(load_cell_lock, Source::Output)
    .position(|lock| lock.as_slice() == lock_script.as_slice())
    .map_or_else(|| Err(Error::InvalidPaymentLockScript),
      |index| load_cell_capacity(index, Source::Output)
        .map_or(Err(Error::Encoding), |result_| Ok(result_)))
}

pub fn load_cell_data_by_type_hash(
  source: Source,
  predicate: &dyn Fn(&[u8]) -> bool,
) -> Option<Vec<u8>> {
  QueryIter::new(load_cell_type_hash, source)
    .position(|type_hash_opt| type_hash_opt.map_or(false, |type_hash| predicate(&type_hash)))
    .map(|index| load_cell_data(index, source).map_or_else(|_| Vec::new(), |data| data))
}

pub fn load_output_type_args_ids(
  slice_start: usize,
  predicate: &dyn Fn(&Script) -> bool,
) -> Vec<u32> {
  QueryIter::new(load_cell_type, Source::Output)
    .filter(|type_opt| parse_type_opt(&type_opt, predicate))
    .filter_map(|type_opt| type_opt.and_then(|type_| parse_type_args_id(type_, slice_start)))
    .collect()
}

fn cell_deps_have_same_issuer_id(issuer_id: &[u8]) -> Result<bool, Error> {
  let type_hash_opt = load_cell_type_hash(0, Source::CellDep)?;
  type_hash_opt.map_or(Ok(false), |_type_hash| {
    Ok(&_type_hash[0..ISSUER_TYPE_ARGS_LEN] == issuer_id)
  })
}

fn cell_deps_have_same_class_type(class_type: &Script) -> Result<bool, Error> {
  let type_opt = load_cell_type(0, Source::CellDep)?;
  type_opt.map_or(Ok(false), |_type| {
    Ok(_type.as_slice() == class_type.as_slice())
  })
}

pub fn cell_deps_and_inputs_have_issuer_or_class_lock(nft_args: &Bytes) -> Result<bool, Error> {
  let cell_dep_lock = load_cell_lock(0, Source::CellDep)?;
  let input_lock = load_cell_lock(0, Source::Input)?;
  if cell_dep_lock.as_slice() == input_lock.as_slice() {
    if cell_deps_have_same_issuer_id(&nft_args[0..ISSUER_TYPE_ARGS_LEN])? {
      return Ok(true);
    }
    let class_type = load_class_type(nft_args);
    if cell_deps_have_same_class_type(&class_type)? {
      return Ok(true);
    }
  }
  Ok(false)
}

pub fn check_group_input_witness_is_none_with_type(type_script: &Script) -> Result<bool, Error> {
  let lock_script: Script = QueryIter::new(load_cell_type, Source::Input)
    .position(|type_opt| type_opt.map_or(false, |type_| type_.as_slice() == type_script.as_slice()))
    .map(|index| load_cell_lock(index, Source::Input).map_or(Err(Error::Encoding), Ok))
    .map_or_else(|| Err(Error::Encoding), |lock_| lock_)?;

  QueryIter::new(load_cell_lock, Source::Input)
    .position(|lock| lock.as_slice() == lock_script.as_slice())
    .map(|index| {
      load_witness_args(index, Source::Input).map_or_else(
        |_| Ok(true),
        |witness_args| Ok(witness_args.lock().to_opt().is_none()),
      )
    })
    .map_or_else(|| Err(Error::Encoding), |result_| result_)
}

pub fn parse_dyn_vec_len(data: &[u8]) -> usize {
  let mut size_buf = [0u8; 2];
  size_buf.copy_from_slice(&data[..]);
  let size = u16::from_be_bytes(size_buf) as usize;
  size + DYN_MIN_LEN
}

pub fn u32_from_slice(data: &[u8]) -> u32 {
  let mut buf = [0u8; 4];
  buf.copy_from_slice(data);
  u32::from_be_bytes(buf)
}

pub fn u64_from_slice(data: &[u8]) -> u64 {
  let mut buf = [0u8; 8];
  buf.copy_from_slice(data);
  u64::from_be_bytes(buf)
}
