mod mock_tx;

use ckb_script::{ScriptGroupType, TransactionScriptsVerifier};
use ckb_types::core::cell::resolve_transaction;
use clap::{arg, command, value_parser};
use mock_tx::{MockTransaction, ReprMockTransaction, Resource};
use serde_json::from_str as from_json_str;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

fn main() {
    let matches = command!()
        .arg(arg!(--"tx-file" <VALUE>).required(true))
        .arg(
            arg!(--"cell-index" <VALUE>)
                .value_parser(value_parser!(u64))
                .required(true),
        )
        .arg(
            arg!(--"cell-type" <VALUE>)
                .value_parser(["input", "output"])
                .required(true),
        )
        .arg(
            arg!(--"script-type" <VALUE>)
                .value_parser(["lock", "type"])
                .required(true),
        )
        .arg(
            arg!(--"vm-sample-times" <VALUE>)
                .value_parser(value_parser!(u8))
                .default_value("5"),
        )
        .get_matches();

    let tx_file = matches.get_one::<String>("tx-file").expect("tx file");
    let vm_sample_times = *matches.get_one::<u8>("vm-sample-times").expect("required");
    let cell_index = *matches.get_one::<u64>("cell-index").expect("cell index");
    let cell_type = matches.get_one::<String>("cell-type").expect("cell type");
    let script_type = matches
        .get_one::<String>("script-type")
        .expect("script type");

    let content = read_to_string(tx_file).expect("read");
    let repr_tx: ReprMockTransaction = from_json_str(&content).expect("json parsing");
    let mock_tx: MockTransaction = repr_tx.into();

    let verifier_resource = Resource::from_mock_tx(&mock_tx).expect("create resource");
    let resolved_tx = resolve_transaction(
        mock_tx.core_transaction(),
        &mut HashSet::new(),
        &verifier_resource,
        &verifier_resource,
    )
    .expect("resolve");
    let verifier = TransactionScriptsVerifier::new(Arc::new(resolved_tx), verifier_resource);

    let (group_type, script_hash) = match (cell_type.as_str(), script_type.as_str()) {
        ("input", "lock") => (
            ScriptGroupType::Lock,
            mock_tx.mock_info.inputs[cell_index as usize]
                .output
                .calc_lock_hash(),
        ),
        ("input", "type") => (
            ScriptGroupType::Type,
            mock_tx.mock_info.inputs[cell_index as usize]
                .output
                .type_()
                .to_opt()
                .expect("cell should have type script")
                .calc_script_hash(),
        ),
        ("output", "type") => (
            ScriptGroupType::Type,
            mock_tx
                .tx
                .raw()
                .outputs()
                .get(cell_index as usize)
                .expect("index out of bound")
                .type_()
                .to_opt()
                .expect("cell should have type script")
                .calc_script_hash(),
        ),
        _ => panic!(
            "Script {} {} {} shall not be executed!",
            cell_type, cell_index, script_type
        ),
    };

    let mut total_runtime = Duration::default();
    let cycle = {
        let a = SystemTime::now();
        let cycle = verifier
            .verify_single(group_type, &script_hash, u64::max_value())
            .expect("run");
        let b = SystemTime::now();
        let d = b.duration_since(a).expect("clock goes backwards");
        total_runtime = total_runtime.checked_add(d).expect("time overflow!");
        cycle
    };

    for _ in 1..vm_sample_times {
        let a = SystemTime::now();
        let sample_cycle = verifier
            .verify_single(group_type, &script_hash, u64::max_value())
            .expect("run");
        let b = SystemTime::now();
        assert_eq!(sample_cycle, cycle);
        let d = b.duration_since(a).expect("clock goes backwards");
        total_runtime = total_runtime.checked_add(d).expect("time overflow!");
    }
    let runtime_nanoseconds = total_runtime.as_nanos() / vm_sample_times as u128;

    println!("{} nanoseconds", runtime_nanoseconds);
}
