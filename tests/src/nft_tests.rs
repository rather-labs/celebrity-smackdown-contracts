use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_testtool::ckb_hash::Blake2bBuilder;
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView,},
    packed::*,
    prelude::*,
};

const MAX_CYCLES: u64 = 70_000_000;

const TYPE: u8 = 1;
const CLASS_TYPE_CODE_HASH: [u8; 32] = [
  178,  41, 227,  89, 116, 182, 82, 136,
   50,  89,  68,  83,  32, 115, 46,  24,
  142, 155, 214,   3,  74,  26, 47, 192,
   28, 109, 121, 199, 250, 107,  6, 242
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

fn create_test_context() -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();

    let nft_bin: Bytes = Loader::default().load_binary("nft-type");
    let nft_out_point = context.deploy_cell(nft_bin);
    let nft_type_script_dep = CellDep::new_builder()
        .out_point(nft_out_point.clone())
        .build();

    // deploy always_success script
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

        //////
    let issuer_bin: Bytes = Loader::default().load_binary("issuer-type");
    let issuer_out_point = context.deploy_cell(issuer_bin);

    let issuer_type_args = hex::decode("157a3633c3477d84b604a25e5fca5ca681762c10").unwrap();
    let issuer_type_script = context
        .build_script(&issuer_out_point, Bytes::from(issuer_type_args.clone()))
        .expect("script");

    // class type script and inputs
    let class_input_data = Bytes::from(hex::decode("0100000000000003E8000200000002000000020000").unwrap());

    let issuer_type_hash: [u8; 32] = issuer_type_script.clone().calc_script_hash().unpack();
    let mut class_type_args = issuer_type_hash[0..20].to_vec();
    let mut args_class_id = 8u32.to_be_bytes().to_vec();
    class_type_args.append(&mut args_class_id);

    let class_aggron_type_script = Script::new_builder()
        .code_hash(CLASS_TYPE_CODE_HASH.pack())
        .args(Bytes::copy_from_slice(&class_type_args[..]).pack())
        .hash_type(Byte::new(TYPE))
        .build();
    let class_cell_dep_aggron_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(2000u64.pack())
            .lock(lock_script.clone())
            .type_(Some(class_aggron_type_script.clone()).pack())
            .build(),
        class_input_data,
    );
    let class_cell_aggron_dep = CellDep::new_builder()
        .out_point(class_cell_dep_aggron_out_point.clone())
        .build();

    // another class type script and inputs
    let class_input_data_2 = Bytes::from(hex::decode("0200000000000001F4000200000002000000020000").unwrap());

    let issuer_type_hash: [u8; 32] = issuer_type_script.clone().calc_script_hash().unpack();
    let mut class_type_args_2 = issuer_type_hash[0..20].to_vec();
    let mut args_class_id_2 = 2u32.to_be_bytes().to_vec();
    class_type_args_2.append(&mut args_class_id_2);

    let class_aggron_type_script_2 = Script::new_builder()
        .code_hash(CLASS_TYPE_CODE_HASH.pack())
        .args(Bytes::copy_from_slice(&class_type_args_2[..]).pack())
        .hash_type(Byte::new(TYPE))
        .build();
    let class_cell_dep_aggron_out_point_2 = context.create_cell(
        CellOutput::new_builder()
            .capacity(2000u64.pack())
            .lock(lock_script.clone())
            .type_(Some(class_aggron_type_script_2.clone()).pack())
            .build(),
        class_input_data_2,
    );
    let class_cell_aggron_dep_2 = CellDep::new_builder()
        .out_point(class_cell_dep_aggron_out_point_2.clone())
        .build();

    // funding cell
    let normal_input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let normal_input = CellInput::new_builder()
        .previous_output(normal_input_out_point.clone())
        .build();
    let inputs = vec![normal_input.clone()];

    // nft type script and inputs
    let mut nft_type_args = class_type_args.clone().to_vec();
    
    let first_input_previous_output = normal_input.previous_output();
    let mut blake2b = Blake2bBuilder::new(32).build();
    blake2b.update(first_input_previous_output.tx_hash().as_slice());
    // Use this cell output index
    let output_index = 1u64;
    blake2b.update(&output_index.to_le_bytes());
    let mut ret = [0; 32];
    blake2b.finalize(&mut ret);

    nft_type_args.append(&mut ret.to_vec());

    let nft_type_script = context
        .build_script(&nft_out_point, Bytes::copy_from_slice(&nft_type_args[..]))
        .expect("script");

    // another nft from same class
    let mut nft_type_args_2 = class_type_args.clone().to_vec();
    
    let first_input_previous_output = normal_input.previous_output();
    let mut blake2b = Blake2bBuilder::new(32).build();
    blake2b.update(first_input_previous_output.tx_hash().as_slice());
    // Use this cell output index
    let output_index = 2u64;
    blake2b.update(&output_index.to_le_bytes());
    let mut ret = [0; 32];
    blake2b.finalize(&mut ret);

    nft_type_args_2.append(&mut ret.to_vec());

    let nft_type_script_2 = context
        .build_script(&nft_out_point, Bytes::copy_from_slice(&nft_type_args_2[..]))
        .expect("script");

    // another nft from another class
    let mut nft_type_args_3 = class_type_args_2.clone().to_vec();
    
    let first_input_previous_output = normal_input.previous_output();
    let mut blake2b = Blake2bBuilder::new(32).build();
    blake2b.update(first_input_previous_output.tx_hash().as_slice());
    // Use this cell output index
    let output_index = 3u64;
    blake2b.update(&output_index.to_le_bytes());
    let mut ret = [0; 32];
    blake2b.finalize(&mut ret);

    nft_type_args_3.append(&mut ret.to_vec());

    let nft_type_script_3 = context
        .build_script(&nft_out_point, Bytes::copy_from_slice(&nft_type_args_3[..]))
        .expect("script");

    // Payment output cell
    let payment_lock_script = Script::new_builder()
        .code_hash(PAYMENT_TYPE_CODE_HASH.pack())
        .args(Bytes::copy_from_slice(&PAYMENT_TYPE_ARGS).pack())
        .hash_type(Byte::new(TYPE))
        .build();
    let payment_cell_output = CellOutput::new_builder()
            .capacity((100000000u64 * 2400).pack())
            .lock(payment_lock_script.clone())
            .build();
 
    let outputs = vec![
      payment_cell_output,
      CellOutput::new_builder()
        .capacity(500u64.pack())
        .lock(lock_script.clone())
        .type_(Some(nft_type_script.clone()).pack())
        .build(),
      CellOutput::new_builder()
        .capacity(500u64.pack())
        .lock(lock_script.clone())
        .type_(Some(nft_type_script_2.clone()).pack())
        .build(),
      CellOutput::new_builder()
        .capacity(500u64.pack())
        .lock(lock_script.clone())
        .type_(Some(nft_type_script_3.clone()).pack())
        .build(),
    ];
        
    let outputs_data: Vec<_> = vec![
      Bytes::from(hex::decode("00").unwrap()),
      Bytes::from(hex::decode("01000100006400000010000001550002616546546660000003898989").unwrap()),
      Bytes::from(hex::decode("010001000064000000100000015500026165465466600000898989").unwrap()),
      Bytes::from(hex::decode("0200010000640000001000000155000261654654666000898989").unwrap()),
    ];

    let mut witnesses = vec![];
    witnesses.push(Bytes::from(hex::decode("5500000010000000550000005500000041000000b69c542c0ee6c4b6d8350514d876ea7d8ef563e406253e959289457204447d2c4eb4e4a993073f5e76d244d2f93f7c108652e3295a9c8d72c12477e095026b9500").unwrap()));

    let cell_deps = vec![lock_script_dep, class_cell_aggron_dep, class_cell_aggron_dep_2, nft_type_script_dep];

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_deps(cell_deps)
        .witnesses(witnesses.pack())
        .build();
    (context, tx)
}

#[test]
fn test_create_nft_cells_success() {
    let (mut context, tx) = create_test_context();

    let tx = context.complete_tx(tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
