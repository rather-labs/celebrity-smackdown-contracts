mod validator;
use self::validator::validate_immutable_nft_fields;
use alloc::vec::Vec;
use ckb_std::{
  ckb_constants::Source,
  ckb_types::{bytes::Bytes, packed::*, prelude::*},
  high_level::{load_cell_data, load_script},
};
use core::result::Result;
use script_utils::{
  class::{Class, CLASS_TYPE_ARGS_LEN},
  error::Error,
  helper::{
    check_group_input_witness_is_none_with_type, count_cells_by_type, load_cell_data_by_type,
    load_class_type, load_output_type_args_ids, Action,
  },
  nft::{Nft, NFT_TYPE_ARGS_LEN},
};

fn check_class_type<'a>(nft_args: &'a Bytes) -> impl Fn(&Script) -> bool + 'a {
  let class_type = load_class_type(nft_args);
  move |type_: &Script| type_.as_slice() == class_type.as_slice()
}

fn check_nft_type<'a>(nft_type: &'a Script) -> impl Fn(&Script) -> bool + 'a {
  let nft_args: Bytes = nft_type.args().unpack();
  move |type_: &Script| {
    let type_args: Bytes = type_.args().unpack();
    type_.code_hash().as_slice() == nft_type.code_hash().as_slice()
      && type_.hash_type().as_slice() == nft_type.hash_type().as_slice()
      && type_args.len() == NFT_TYPE_ARGS_LEN
      && type_args[0..CLASS_TYPE_ARGS_LEN] == nft_args[0..CLASS_TYPE_ARGS_LEN]
  }
}

fn load_nft_data(source: Source) -> Result<Vec<u8>, Error> {
  load_cell_data(0, source).map_err(|_| Error::NFTDataInvalid)
}

fn parse_nft_action(nft_type: &Script) -> Result<Action, Error> {
  let nft_inputs_count = count_cells_by_type(Source::Input, &check_nft_type(nft_type));
  if nft_inputs_count == 0 {
    return Ok(Action::Create);
  }

  let nft_outputs_count = count_cells_by_type(Source::Output, &check_nft_type(nft_type));
  if nft_inputs_count == 1 && nft_outputs_count == 0 {
    return Ok(Action::Destroy);
  }

  if nft_inputs_count == nft_outputs_count {
    return Ok(Action::Update);
  }
  Err(Error::NFTCellsCountError)
}

fn handle_creation(nft_type: &Script) -> Result<(), Error> {
  let nft_args: Bytes = nft_type.args().unpack();
  let class_inputs_count = count_cells_by_type(Source::Input, &check_class_type(&nft_args));
  if class_inputs_count != 1 {
    return Err(Error::ClassCellsCountError);
  }

  let load_class = |source| match load_cell_data_by_type(source, &check_class_type(&nft_args)) {
    Some(data) => Ok(Class::from_data(&data)?),
    None => Err(Error::ClassDataInvalid),
  };
  let input_class = load_class(Source::Input)?;
  let output_class = load_class(Source::Output)?;

  if output_class.nft_count <= input_class.nft_count {
    return Err(Error::ClassIssuedInvalid);
  }

  let outputs_token_ids = load_output_type_args_ids(CLASS_TYPE_ARGS_LEN, &check_nft_type(nft_type));
  let nft_outputs_increased_count = (output_class.nft_count - input_class.nft_count) as usize;
  if nft_outputs_increased_count != outputs_token_ids.len() {
    return Err(Error::NFTCellsCountError);
  }

  let mut class_cell_token_ids = Vec::new();
  for token_id in input_class.nft_count..output_class.nft_count {
    class_cell_token_ids.push(token_id);
  }

  if outputs_token_ids != class_cell_token_ids {
    return Err(Error::NFTTokenIdIncreaseError);
  }

  Ok(())
}

fn handle_update(nft_type: &Script) -> Result<(), Error> {
  // Disable anyone-can-pay lock
  if check_group_input_witness_is_none_with_type(nft_type)? {
    return Err(Error::GroupInputWitnessNoneError);
  }
  let nft_data = (
    load_nft_data(Source::GroupInput)?,
    load_nft_data(Source::GroupOutput)?,
  );
  let nfts = (
    Nft::from_data(&nft_data.0[..])?,
    Nft::from_data(&nft_data.1[..])?,
  );
  validate_immutable_nft_fields(&nfts)?;

  Ok(())
}

fn handle_destroying(nft_type: &Script) -> Result<(), Error> {
  // Disable anyone-can-pay lock
  if check_group_input_witness_is_none_with_type(nft_type)? {
    return Err(Error::GroupInputWitnessNoneError);
  }

  Ok(())
}

pub fn main() -> Result<(), Error> {
  let nft_type = load_script()?;
  let nft_args: Bytes = nft_type.args().unpack();
  if nft_args.len() != NFT_TYPE_ARGS_LEN {
    return Err(Error::TypeArgsInvalid);
  }

  match parse_nft_action(&nft_type)? {
    Action::Create => handle_creation(&nft_type),
    Action::Update => handle_update(&nft_type),
    Action::Destroy => handle_destroying(&nft_type),
  }
}
