use crate::debug_log;
use crate::error::Error;
use alloc::vec::Vec;
use ckb_std::ckb_constants::Source;
use witness_parser::WitnessesParserV1;

use das_core::constants::ScriptType;
use das_core::util::{find_only_cell_by_type_id, hex_string};
use das_types::constants::DataType;
use das_types::constants::TypeScript;
use das_types::packed;
use das_types::packed::{AccountApprovalTransfer, AccountCellData, Hash};
use das_types::prelude::Entity;
use witness_parser::traits::WitnessQueryable;
use witness_parser::types::CellMeta;

pub fn get_type_id_by_type_script(type_script: TypeScript) -> Result<Vec<u8>, Error> {
    debug_log!("get type id of {:?}", &type_script);
    let parser = WitnessesParserV1::get_instance();
    if !parser.is_inited() {
        parser
            .init()
            .map_err(|err| {
                debug_log!("Error: witness parser init failed, {:?}", err);
                das_core::error::ErrorCode::WitnessDataDecodingError
            })
            .unwrap();
        debug_log!("WitnessesParserV1::init() success");
    }else {
        debug_log!("WitnessesParserV1::init() already inited");
    }
    debug_log!("WitnessesParserV1::get_instance() success");

    let type_id = parser
        .get_type_id(type_script.clone())
        .map_err(|err| {
            debug_log!(
                "Error: witness parser get type id of {:?} failed, {:?}",
                &type_script,
                err
            );
            das_core::error::ErrorCode::WitnessDataDecodingError
        })
        .unwrap();

    debug_log!("{:?} type id is {:?}", &type_script, hex_string(&type_id));

    // let idx = match type_script {
    //     TypeScript::AccountCellType => {0}
    //     TypeScript::SubAccountCellType => {1}
    //     TypeScript::DPointCellType => {2}
    //     TypeScript::EIP712Lib => {3}
    //     TypeScript::BalanceCellType => {4}
    //     _ => {
    //         unreachable!();
    //     }
    // };
    // let type_id = crate::constants::decode_hex("type id", TYPE_ID_TABLE_TYPE[idx]);
    //
    let type_id_vec = type_id.to_vec();
    Ok(type_id_vec)
}
//Some wrappers for get_type_id_by_type_script
// pub fn get_balance_cell_type_id() -> Result<Vec<u8>, Error> {
//     get_type_id_by_type_script(TypeScript::BalanceCellType)
// }
pub fn get_sub_account_cell_type_id() -> Result<Vec<u8>, Error> {
    get_type_id_by_type_script(TypeScript::SubAccountCellType)
}

pub fn get_account_cell_type_id() -> Result<Vec<u8>, Error> {
    get_type_id_by_type_script(TypeScript::AccountCellType)
}

pub fn get_dpoint_cell_type_id() -> Result<Vec<u8>, Error> {
    get_type_id_by_type_script(TypeScript::DPointCellType)
}
pub fn get_reverse_record_root_cell_type_id() -> Result<Vec<u8>, Error> {
    get_type_id_by_type_script(TypeScript::ReverseRecordRootCellType)
}
pub fn get_first_account_cell_index() -> Result<usize, Error> {
    let account_cell_type_id = get_account_cell_type_id()?;
    let index = find_only_cell_by_type_id(
        ScriptType::Type,
        Hash::from_slice(account_cell_type_id.as_slice())
            .unwrap()
            .as_reader(),
        Source::Input,
    )?;
    //let input_account_cells = util::load_self_cells_in_inputs()?;
    Ok(index)
}

pub fn get_input_approval() -> Result<AccountApprovalTransfer, Error> {
    let account_cell_witness = get_account_cell_witness()?;
    let account_cell_witness_reader = account_cell_witness.as_reader();
    let input_approval_params = AccountApprovalTransfer::from_compatible_slice(
        account_cell_witness_reader.approval().params().raw_data(),
    )
    .map_err(|e| {
        debug_log!("Decoding AccountCell.witness.approval.params failed: {}", e);
        return Error::WitnessError;
    })
    .unwrap();
    //let input_approval_reader = input_approval_params.as_reader();
    Ok(input_approval_params)
}

pub fn get_account_cell_witness() -> Result<packed::AccountCellData, Error> {
    debug_log!("get_account_cell_witness");
    let witness_parser = WitnessesParserV1::get_instance();
    debug_log!("WitnessesParserV1::get_instance() success");

    // let account_cell_witness = witness_parser
    //     .get_entity_by_data_type::<packed::AccountCellData>(DataType::AccountCellData)
    //     .map_err(|e| {
    //         debug_log!("WitnessParserV1 get_entity_by_data_type error: {:?}", e);
    //         return Error::WitnessError;
    //     })
    //     .unwrap();
    let cell_meta = CellMeta {
        index: 0,
        source: das_types::constants::Source::Input,
    };

    let account_cell_witness = witness_parser
        .get_entity_by_cell_meta::<AccountCellData>(cell_meta)
        .map_err(|e| {
            debug_log!("WitnessParserV1 get_entity_by_data_type error: {:?}", e);
            return Error::WitnessError;
        })
        .unwrap();
    Ok(account_cell_witness)
}
