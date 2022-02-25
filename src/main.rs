#![allow(dead_code,unused_variables)]

mod parser;
mod core_rpc;
mod tests;
mod transaction;
mod ecc;

fn main() {

    tests::run();

    // Gets command line parameters (txid hash rawtransaction)
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 4 {
        // TODO: parameter checks (valid length, valid hex)
        println!("Generating...");
        let (cov_tx, cpfp_tx) = generate_next_cov_tx_and_cpfp(&args[1], &args[2], &args[3]);
        println!("Covenant tx:\n{cov_tx}");
        println!("Fee-bumping cpfp tx:\n{cpfp_tx}");
        println!("DONE!");
     }
     else {
        println!("Please run this with the following parameters in hex: covenant_txid spacechain_hash cpfp_rawtransaction");
    }
}

// Main function, outputs the cov_tx and cpfp_tx with user added hash and fee
fn generate_next_cov_tx_and_cpfp(prev_txid: &str, hash: &str, rawtransaction: &str) -> (String, String) {
    
    // Find the next covenant tx (based on the txid of the previous one)
    const KEY_STRING: &str = "eb445ec7e0fd814db1e84622cddad9cd30154ee22bc6c2a4a61f6287be39f2d2";
    const INPUT_TXID: &str = "60c31751818bd4410eed84b1c9047863206cce2c7d4d610ce5841c4195ba6c3b"; // signet now, was regtest "2715afb15d8f92028de0fd98e68e48ee496ac83f5c9dbf031a0a6a21a7e9cd59";
    const INPUT_VOUT: u64 = 1; // Note this isn't a fixed value
    const INPUT_SATOSHIS: u64 = 100_000; // signet now, was 10*100_000_000;
    let (cov_tx_string, cov_txid) = find_covenant_tx(KEY_STRING, INPUT_TXID.to_string(), INPUT_VOUT, INPUT_SATOSHIS, prev_txid);

    // Build the cpfp input and op_return output
    let cpfp_tx = build_feebump_tx(&cov_txid, 1, 800, hash);

    // Take the rawtransaction (assumed 1 input 1 output) and merge it with the above
    let mut raw_tx = parser::decode_from_hex(rawtransaction);
    raw_tx.inputs.push(cpfp_tx.inputs[0].clone());
    raw_tx.outputs.push(cpfp_tx.outputs[0].clone());
    let merged_raw_tx_string = parser::encode_to_hex(&raw_tx);

    return (cov_tx_string, merged_raw_tx_string)
}

// Changes the transaction object and outputs a signed hex tx
fn sign_tx(tx: &mut transaction::Content, input_index: u64, input_scriptpubkey: &str, key: &ecc::ECC) -> String {
    let sh_all = parser::get_sighash_all_data(&tx, input_index, input_scriptpubkey);
    let sig = key.sign_ecdsa_der(&sh_all) + "01"; // 01 == sighash_all
    tx.inputs[input_index as usize].scriptsig = parser::get_length_prefixed_string(&sig); // TODO: doesn't work if more than sig is needed
    tx.txid = parser::tx_to_txid(&tx);
    let tx_string = parser::encode_to_hex(&tx);
    return tx_string
}

// Builds a partial transaction that can pay for the cpfp (not needed if rawtransaction is used instead)
fn build_paying_tx(input_txid: &str, input_vout: u64, input_amount: u64, output_scriptpubkey: &str, fee: u64) -> transaction::Content {
    let (mut inputs, mut outputs) = (vec![], vec![]);
    
    inputs.push(transaction::Input {
        txid:       input_txid.to_string(),
        vout:       input_vout,
        scriptsig:  String::new(), // still needs a sig later
        sequence:   0
    });

    outputs.push(transaction::Output {
        value:         input_amount - fee,
        scriptpubkey:  output_scriptpubkey.to_string()
    });
    
    let mut tx = transaction::Content {
        txid:       String::new(), // calculated after tx is complete
        version:    2, // needs to be >1 for op_csv
        locktime:   0,
        inputs,
        outputs
    };

    tx.txid = parser::tx_to_txid(&tx);

    return tx
}

fn build_feebump_tx(txid: &str, vout: u64, amount: u64, output_hex_hash: &str) -> transaction::Content {
    let (mut inputs, mut outputs) = (vec![], vec![]);
    
    inputs.push(transaction::Input {
        txid:       txid.to_string(),
        vout:       vout,
        scriptsig:  parser::get_length_prefixed_string(&build_bump_script()), // satisfy the p2sh by revealing the lock script
        sequence:   0 // script forces this to 0
    });

    outputs.push(transaction::Output {
        value:         amount, // - 104 - 104,
        scriptpubkey: "6a".to_string() + &parser::get_length_prefixed_string(&output_hex_hash) // Note tx may be too small without data (non-standard)
    });
    
    let mut tx = transaction::Content {
        txid:       String::new(), // calculated after tx is complete
        version:    2, // needs to be >1 for op_csv
        locktime:   0,
        inputs,
        outputs
    };

    tx.txid = parser::tx_to_txid(&tx);

    return tx
}

// Generates the covenant transactions (needs to be pre-calculated and published instead of key)
fn generate_covenant_tx_sequence(key_string: &str, mut input_txid: String, mut input_vout: u64, mut input_satoshis: u64, reps: u64) -> Vec<String> {
    const COST: u64 = 2000;
    if input_satoshis < COST*reps { panic!("Insufficient funds to generate the desired number of transactions. Requires {} sats", COST*reps); }
    let mut covenant_tx_sequence = vec![];
    let key = ecc::ECC::new(key_string);
    //let script = build_covenant_script(&key.get_pk_string());
    //let p2sh_address = parser::script_to_p2sh_address(&script);
    //println!("{p2sh_address}"); // 2NEcniP26o4oF2jUHgTAcncJnKt653gxrLw
    for _ in 0..reps {
        let tx = build_covenant_tx(&input_txid, input_vout, input_satoshis, &key);
        let tx_string = parser::encode_to_hex(&tx);
        covenant_tx_sequence.push(tx_string);
        input_txid = tx.txid.clone();
        input_vout = 0;
        input_satoshis -= COST;
    }
    return covenant_tx_sequence
}

// Gets the next transaction for use in the covenant, as well as its txid
fn find_covenant_tx(key_string: &str, mut input_txid: String, mut input_vout: u64, mut input_satoshis: u64, target_txid: &str) -> (String, String) {
    const COST: u64 = 2000;
    let key = ecc::ECC::new(key_string);
    //let script = build_covenant_script(&key.get_pk_string());
    //let p2sh_address = parser::script_to_p2sh_address(&script);
    //println!("{p2sh_address}"); // 2NEcniP26o4oF2jUHgTAcncJnKt653gxrLw
    let mut bool = false;
    for _ in 0..52560 { // year
        let tx = build_covenant_tx(&input_txid, input_vout, input_satoshis, &key);
        let tx_string = parser::encode_to_hex(&tx);
        if bool { return (tx_string, tx.txid.clone()) };
        input_txid = tx.txid.clone();
        if input_txid == target_txid { bool = true };
        input_vout = 0;
        if input_satoshis < COST { panic!("Couldn't find target txid (or ran out of coins)") }
        input_satoshis -= COST;
    }
    panic!("Couldn't find target txid")
}

// The main bmm script that's in use
fn build_covenant_script(pubkey_hex_string: &str) -> String {
    return parser::get_length_prefixed_string(&pubkey_hex_string) + "ad" + "51" + "b2"
} // OP_PUSHBYTES_XX <Pubkey> OP_CHECKSIGVERIFY OP_PUSHNUM_1 OP_CSV

fn build_bump_script() -> String {
    return String::new() + "00" + "b2" + "8b"
} // OP_0 OP_CSV OP_1ADD (forces RBF)

// Generates the covenant tx (note, input and output are assumed to have the same script, even the 1st input)
fn build_covenant_tx(input_txid: &str, input_vout: u64, input_satoshis: u64, key: &ecc::ECC) -> transaction::Content {
    let (mut inputs, mut outputs) = (vec![], vec![]);
    let (dust_limit, fee) = (800, 1200); // TODO: shave down these numbers (573 for p2sh dust?)
    let covenant_script = build_covenant_script(&key.get_pk_string());
    let p2sh_script = parser::script_to_p2sh_script(&covenant_script);
    
    inputs.push(transaction::Input {
        txid:       input_txid.to_string(),
        vout:       input_vout,
        scriptsig:  String::new(), // will be sig + covenant_script
        sequence:   1 // matches script relative locktime of 1 block
    });

    outputs.push(transaction::Output {
        value:         input_satoshis - dust_limit - fee,
        scriptpubkey:  p2sh_script.clone()
    });

    outputs.push(transaction::Output {
        value:         dust_limit,
        scriptpubkey:  parser::script_to_p2sh_script(&build_bump_script()) 
    });

    //let p2sh_address = parser::script_to_p2sh_address(&"00b28b");
    //println!("{p2sh_address}"); // 2MzHTWrk6TpuPAauCaWcPNpEs4Q9VYW6iCQ

    let mut tx = transaction::Content {
        txid:       String::new(), // calculated after sig is obtained
        version:    2, // needs to be >1 for op_csv
        locktime:   0,
        inputs,
        outputs
    };
    // TODO: signing should not be done inside this function
    let sighash_all_data = parser::get_sighash_all_data(&tx, 0, &covenant_script); // Note: NOT p2sh_script
    let sig = key.sign_ecdsa_der(&sighash_all_data) + "01"; // sighash flag needs to be added
    tx.inputs[0].scriptsig = parser::get_length_prefixed_string(&sig) + &parser::get_length_prefixed_string(&covenant_script);
    tx.txid = parser::tx_to_txid(&tx);

    return tx
}