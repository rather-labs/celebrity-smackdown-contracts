use alloc::vec::Vec;
use blake2b_rs::Blake2bBuilder;
use ckb_std::{
  ckb_constants::Source,
  ckb_types::{bytes::Bytes, packed::*, prelude::*},
  high_level::{load_cell_data, load_input, load_script},
};
use core::result::Result;
use script_utils::{
  error::Error,
  helper::{check_group_input_witness_is_none_with_type, count_cells_by_type, Action},
  metadata::{Metadata, METADATA_TYPE_ARGS_LEN},
};

fn load_metadata_data(source: Source) -> Result<Vec<u8>, Error> {
  load_cell_data(0, source).map_err(|_| Error::MetadataDataInvalid)
}

fn parse_metadata_action(metadata_type: &Script) -> Result<Action, Error> {
  let count_cells = |source| {
    count_cells_by_type(source, &|type_: &Script| {
      type_.as_slice() == metadata_type.as_slice()
    })
  };
  let metadata_cells_count = (count_cells(Source::Input), count_cells(Source::Output));
  match metadata_cells_count {
    (0, 1) => Ok(Action::Create),
    (1, 1) => Ok(Action::Update),
    (1, 0) => Ok(Action::Destroy),
    _ => Err(Error::MetadataCellsCountError),
  }
}

fn handle_creation(metadata_type: &Script) -> Result<(), Error> {
  let first_input = load_input(0, Source::Input)?;
  let first_input_previous_output = first_input.previous_output();
  let mut blake2b = Blake2bBuilder::new(32).build();
  blake2b.update(first_input_previous_output.tx_hash().as_slice());
  blake2b.update(first_input_previous_output.index().as_slice());
  let mut ret = [0; 32];
  blake2b.finalize(&mut ret);

  let metadata_args: Bytes = metadata_type.args().unpack();
  if metadata_args[..] != ret[0..METADATA_TYPE_ARGS_LEN] {
    return Err(Error::TypeArgsInvalid);
  }
  let _metadata = Metadata::from_data(&load_metadata_data(Source::GroupOutput)?[..])?;
  
  Ok(())
}

fn handle_update(metadata_type: &Script) -> Result<(), Error> {
  // Disable anyone-can-pay lock
  if check_group_input_witness_is_none_with_type(metadata_type)? {
    return Err(Error::GroupInputWitnessNoneError);
  }
  let load_metadata = |source| Metadata::from_data(&load_metadata_data(source)?[..]);
  let input_metadata = load_metadata(Source::GroupInput)?;
  let output_metadata = load_metadata(Source::GroupOutput)?;
  if !input_metadata.immutable_equal(&output_metadata) {
    return Err(Error::MetadataImmutableFieldsNotSame);
  }
  Ok(())
}

fn handle_destroying(metadata_type: &Script) -> Result<(), Error> {
  // Disable anyone-can-pay lock
  if check_group_input_witness_is_none_with_type(metadata_type)? {
    return Err(Error::GroupInputWitnessNoneError);
  }
  Ok(())
}

pub fn main() -> Result<(), Error> {
  let metadata_type = load_script()?;
  let metadata_args: Bytes = metadata_type.args().unpack();
  if metadata_args.len() != METADATA_TYPE_ARGS_LEN {
    return Err(Error::TypeArgsInvalid);
  }

  match parse_metadata_action(&metadata_type)? {
    Action::Create => handle_creation(&metadata_type),
    Action::Update => handle_update(&metadata_type),
    Action::Destroy => handle_destroying(&metadata_type),
  }
}
