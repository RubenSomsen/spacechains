use super::transaction;
use super::parser;
use sha2::{Sha256,Digest};

pub fn run() {
    println!("Tests running");

    // tests sighash_all creation from transaction
    let sighash_all_data = parser::get_sighash_all_data(&test_tx_for_sighash_all(),0, &"76a9144299ff317fcd12ef19047df66d72454691797bfc88ac".to_string());
    let sighash_all = hex::encode(Sha256::digest(Sha256::digest(hex::decode(&sighash_all_data).unwrap())));
    assert_eq!(sighash_all,"a6b4103f527dfe43dfbadf530c247bac8a98b7463c7c6ad38eed97021d18ffcb");

    // Turns transaction hex into an object with readable fields
    let transaction = "0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600";
    let tx_content_a = parser::decode_from_hex(transaction);

    /* TODO: skip rpc commands if bitcoind isn't running

    // Does the same thing as above, but via Bitcoin Core rpc commands
    let tx_content_b = core_rpc::decoderawtransaction(transaction);

    // Checks if the parser produced the same result as Bitcoin Core
    assert!(tx_content_a.compare(&tx_content_b)); */

    // Converts the transaction object back into hex (with equality check)
    let content_to_hex = parser::encode_to_hex(&tx_content_a);
    assert!(transaction==content_to_hex);

    // Turns an unlock script into a valid p2sh address: 3CK4fEwbMP7heJarmU4eqA3sMbVJyEnU3V
    let scriptpubkey = "5121022afc20bf379bc96a2f4e9e63ffceb8652b2b6a097f63fbee6ecec2a49a48010e2103a767c7221e9f15f870f1ad9311f5ab937d79fcaeee15bb2c722bca515581b4c052ae";
    let p2sh_address = parser::script_to_p2sh_address(scriptpubkey); // 3CK4fEwbMP7heJarmU4eqA3sMbVJyEnU3V (prefix 05, not testnet!)

    println!("Tests successful");
}

pub fn test_tx_for_sighash_all() -> transaction::Content {
    let (mut inputs, mut outputs) = (vec![], vec![]);
    
    let input = transaction::Input {
        txid:       "4ba5cfbbeb418055e412682dddb01ccec683a80dd9e12792a273f3b20d4a99b7".to_string(),
        vout:       0,
        scriptsig:  String::new(),
        sequence:   u64::MAX
    };
    inputs.push(input);

    let output = transaction::Output {
        value:         15000,
        scriptpubkey:  "76a914b3e2819b6262e0b1f19fc7229d75677f347c91ac88ac".to_string()
    };
    outputs.push(output);

    return transaction::Content {
        txid:       String::new(),
        version:    1,
        locktime:   0,
        inputs,
        outputs
    }
}