mod validator;
use self::validator::validate_immutable_nft_fields;
use alloc::vec::Vec;
use blake2b_rs::Blake2bBuilder;
use ckb_std::{
  ckb_constants::Source,
  ckb_types::{bytes::Bytes, packed::*, prelude::*},
  high_level::{load_cell_data, load_script, QueryIter, load_cell_type, load_input, load_cell_occupied_capacity},
};
use core::result::Result;
use script_utils::{
  class::{Class, CLASS_TYPE_ARGS_LEN},
  issuer::ISSUER_TYPE_ARGS_LEN,
  error::Error,
  helper::{
    check_group_input_witness_is_none_with_type, count_cells_by_type, load_cell_data_by_type,
    load_class_type, Action,
    load_payment_cell_capacity,
  },
  nft::{Nft, NFT_TYPE_ARGS_LEN},
};

fn parse_type_opt(type_opt: &Option<Script>, predicate: &dyn Fn(&Script) -> bool) -> bool {
  match type_opt {
    Some(type_) => predicate(type_),
    None => false,
  }
}

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

fn get_cell_output_index_by_type(nft_type: &Script) -> Result<usize, Error> {
  QueryIter::new(load_cell_type, Source::Output)
    .position(|type_opt| {
      match type_opt {
        Some(type_) => {
          type_.code_hash().as_slice() == nft_type.code_hash().as_slice()
          && type_.hash_type().as_slice() == nft_type.hash_type().as_slice()
          && type_.args().as_slice() == nft_type.args().as_slice()
        },
        None => false,
      }
    })
    .map_or(Err(Error::Encoding), |index| Ok(index))
}

fn get_cell_occupied_capacity_by_type(nft_type: &Script) -> u64 {
  let index = QueryIter::new(load_cell_type, Source::Output)
    .position(|type_opt| {
      match type_opt {
        Some(type_) => {
          type_.code_hash().as_slice() == nft_type.code_hash().as_slice()
          && type_.hash_type().as_slice() == nft_type.hash_type().as_slice()
          && type_.args().as_slice() == nft_type.args().as_slice()
        },
        None => false,
      }
    }).unwrap();
  load_cell_occupied_capacity(index, Source::Output).unwrap()
}

fn check_issuer_type<'a>(nft_type: &'a Script) -> impl Fn(&Script) -> bool + 'a {
  let nft_args: Bytes = nft_type.args().unpack();
  move |type_: &Script| {
    let type_args: Bytes = type_.args().unpack();
    type_.code_hash().as_slice() == nft_type.code_hash().as_slice()
      && type_.hash_type().as_slice() == nft_type.hash_type().as_slice()
      && type_args.len() == NFT_TYPE_ARGS_LEN
      && type_args[0..ISSUER_TYPE_ARGS_LEN] == nft_args[0..ISSUER_TYPE_ARGS_LEN]
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
  // Check type Script Args
  // Use first input output
  let first_input = load_input(0, Source::Input)?;
  let first_input_previous_output = first_input.previous_output();
  let mut blake2b = Blake2bBuilder::new(32).build();
  blake2b.update(first_input_previous_output.tx_hash().as_slice());
  // Use this cell output index
  let output_index = get_cell_output_index_by_type(nft_type)?;
  blake2b.update(&output_index.to_le_bytes());
  let mut ret = [0; 32];
  blake2b.finalize(&mut ret);

  // Check that the last 32 bytes is the expected hash
  let nft_args: Bytes = nft_type.args().unpack();
  if nft_args[24..56] != ret[0..32] {
    return Err(Error::TypeArgsInvalid);
  }

  // Check the nft version is the same as class
  let nft = Nft::from_data(&load_nft_data(Source::GroupOutput)?[..])?;
  // Load data from class cell
  let class_data = Class::from_data(&load_cell_data_by_type(Source::CellDep, &check_class_type(&nft_args)).unwrap())?;
  if class_data.version != nft.version {
    return Err(Error::NFTVersionNotSameWithClass);
  }
  
  // Get all nfts from issuer cell
  let output_nft_types = QueryIter::new(load_cell_type, Source::Output)
    .filter(|type_opt| parse_type_opt(&type_opt, &check_issuer_type(nft_type)));
  
  let mut total_cost: u64 = 0;
  let mut minted_nfts_total_occupied_capacity: u64 = 0;
  for output_nft_type in output_nft_types {
    let nft_type_script = output_nft_type.unwrap();
    let nft_args: Bytes = nft_type_script.args().unpack();

    // Check the class dependency exists for every output nft
    let class_inputs_count = count_cells_by_type(Source::CellDep, &check_class_type(&nft_args));
    if class_inputs_count != 1 {
      return Err(Error::ClassCellsCountError);
    }

    // Load data from class cell
    let class_data = Class::from_data(&load_cell_data_by_type(Source::CellDep, &check_class_type(&nft_args)).unwrap())?;

    // convert cost from CKB to Shannon
    total_cost += class_data.cost * 100000000;
    minted_nfts_total_occupied_capacity += get_cell_occupied_capacity_by_type(&nft_type_script);

  }

  // Load payment cell capacity
  let payment_cell_capacity = load_payment_cell_capacity()?;

  // Check the cost of all nfts is being sent to seller address
  if payment_cell_capacity < (total_cost-minted_nfts_total_occupied_capacity) {
    return Err(Error::PaymentNotEnough);
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
