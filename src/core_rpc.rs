use serde_json::{Value};
use super::transaction;

// Runs the relevant rpc command and parses it
pub fn decoderawtransaction(transaction_hex_string: &str) -> transaction::Content {
    let rpc_tx = bitcoin_cli(vec!["decoderawtransaction", transaction_hex_string]);
    let rpc_inputs = rpc_tx["vin"].as_array().unwrap();
    let rpc_outputs = rpc_tx["vout"].as_array().unwrap();

    let (mut inputs, mut outputs) = (vec![], vec![]);

    for i in 0..rpc_inputs.len() {
        let i = &rpc_inputs[i];
        let input = transaction::Input {
            txid:       i["txid"].as_str().unwrap().to_string(),
            vout:       i["vout"].as_u64().unwrap(),
            scriptsig:  i["scriptSig"]["hex"].as_str().unwrap().to_string(),
            sequence:   i["sequence"].as_u64().unwrap()
        };
        inputs.push(input);
    }

    for i in 0..rpc_outputs.len() {
        let i = &rpc_outputs[i];
        let output = transaction::Output {
            value:         (i["value"].as_f64().unwrap()*100_000_000.0) as u64,
            scriptpubkey:   i["scriptPubKey"]["hex"].as_str().unwrap().to_string()
        };
        outputs.push(output);
    }

    return transaction::Content {
        txid:       rpc_tx["txid"].as_str().unwrap().to_string(),
        version:    rpc_tx["version"].as_u64().unwrap(), 
        locktime:   rpc_tx["locktime"].as_u64().unwrap(), 
        inputs,
        outputs
    }
}

// Run bitcoin_cli rpc commands (requires bitcoind running)
fn bitcoin_cli(args: Vec<&str>) -> Value {
    use std::process::Command;
    let output = Command::new("C:/Program Files/Bitcoin/daemon/bitcoin-cli.exe")
                .args(args)      // TODO: don't hard code, make this work on any device
                .output()
                .expect("failed to execute process");
    let json = &String::from_utf8_lossy(&output.stdout);
    let parsed_output: Value = serde_json::from_str(json).unwrap();
    return parsed_output
}