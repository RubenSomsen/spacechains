use hex;
use sha2::{Sha256, Digest};
use bs58;
use ripemd::{Ripemd160};
use super::transaction;
#[path = "./bytestream.rs"] mod bytestream;

// NOTE: missing weight, script parsing

// Turns a lock script into a hash for use in a p2sh output
pub fn script_to_p2sh_hash160(script_hex_string: &str) -> String {
    let hash1 = Sha256::digest(hex::decode(script_hex_string).unwrap());
    let mut hasher = Ripemd160::new();
    hasher.update(hash1);
    let hash2 = hasher.finalize();
    return hex::encode(hash2).to_string()
}

// Turns a lock script into a valid p2sh address
pub fn script_to_p2sh_address(script_hex_string: &str) -> String {
    let address = "C4".to_string() + &script_to_p2sh_hash160(script_hex_string); // C4 = testnet, 05 = mainnet
    let hash3 = Sha256::digest(Sha256::digest(hex::decode(&address).unwrap()));
    let checksum = &hex::encode(hash3)[0..8];
    let base58 = bs58::encode(hex::decode(address + checksum).unwrap()).into_string();
    return base58
}

// Turns an unlock script into a p2sh unlock script
pub fn script_to_p2sh_script(script_hex_string: &str) -> String {
    return "a9".to_string() + "14" + &script_to_p2sh_hash160(script_hex_string) + "87";
}  // OP_HASH160 OP_PUSHBYTES20 <Hash160> OP_EQUAL

// Turns transaction hex into an object with readable fields
pub fn decode_from_hex(transaction_hex_string: &str) -> transaction::Content {
    let mut tx = bytestream::Bytestream::new(transaction_hex_string);
    let hash = Sha256::digest(Sha256::digest(hex::decode(transaction_hex_string).unwrap()));
    let txid = bytestream::Bytestream::convert_endian(&hex::encode(hash));
    let version = bytestream::Bytestream::bytes_to_u64(&tx.get_bytes(4, true));
    let no_of_inputs = tx.get_varint();
    let mut inputs = vec![];
    for _ in 0..no_of_inputs {
        let txid = hex::encode(tx.get_bytes(32, true));
        let vout = bytestream::Bytestream::bytes_to_u64(&tx.get_bytes(4, true));
        let scriptsig_size = tx.get_varint();
        let scriptsig = hex::encode(tx.get_bytes(scriptsig_size, false));
        let sequence = bytestream::Bytestream::bytes_to_u64(&tx.get_bytes(4, true));
        inputs.push(transaction::Input { txid, vout, scriptsig, sequence });
    }
    let no_of_outputs = tx.get_varint();
    let mut outputs = vec![];
    for _ in 0..no_of_outputs {
        let value = bytestream::Bytestream::bytes_to_u64(&tx.get_bytes(8, true));
        let scriptpubkey_size = tx.get_varint();
        let scriptpubkey = hex::encode(tx.get_bytes(scriptpubkey_size, false));
        outputs.push(transaction::Output { value, scriptpubkey });
    }
    let locktime = bytestream::Bytestream::bytes_to_u64(&tx.get_bytes(4, true));
    return transaction::Content { txid, version, locktime, inputs, outputs }
}

// Turns a transaction object into a hex transaction string
pub fn encode_to_hex(tx: &transaction::Content) -> String {
    let mut s = String::new();
    s += &hex::encode((tx.version as u32).to_le_bytes());
    s += &to_hex_var_int(tx.inputs.len() as u64);
    for i in 0..tx.inputs.len() {
        let i = &tx.inputs[i];
        s += &bytestream::Bytestream::convert_endian(&i.txid);
        s += &hex::encode((i.vout as u32).to_le_bytes());
        s += &get_length_prefixed_string(&i.scriptsig);
        s += &hex::encode((i.sequence as u32).to_le_bytes());
        
    }
    s += &to_hex_var_int(tx.outputs.len() as u64);
    for i in 0..tx.outputs.len() {
        let i = &tx.outputs[i];
        s += &hex::encode(i.value.to_le_bytes());
        s += &get_length_prefixed_string(&i.scriptpubkey);
    }
    s += &hex::encode((tx.locktime as u32).to_le_bytes());
    return s
}

// Gets the txid of a transaction object
pub fn tx_to_txid(tx: &transaction::Content) -> String {
    let transaction_hex_string = encode_to_hex(tx);
    let hash = Sha256::digest(Sha256::digest(hex::decode(transaction_hex_string).unwrap()));
    let txid = bytestream::Bytestream::convert_endian(&hex::encode(hash));
    return txid
}

// Calculates the (unhashed) sighash_all data for a transaction object
pub fn get_sighash_all_data(tx: &transaction::Content, input_index: u64, input_scriptpubkey: &str) -> String {
    let mut s = String::new();
    s += &hex::encode((tx.version as u32).to_le_bytes());
    s += &to_hex_var_int(tx.inputs.len() as u64);
    for i0 in 0..tx.inputs.len() {
        let i = &tx.inputs[i0];
        s += &bytestream::Bytestream::convert_endian(&i.txid);
        s += &hex::encode((i.vout as u32).to_le_bytes());
        if i0 as u64==input_index { // target input gets scriptpubkey
            s += &get_length_prefixed_string(&input_scriptpubkey);
        }
        else { s += "00" } // else the scriptsig remains empty (length of 0, hence 0x00)
        s += &hex::encode((i.sequence as u32).to_le_bytes());
        
    }
    s += &to_hex_var_int(tx.outputs.len() as u64);
    for i in 0..tx.outputs.len() {
        let i = &tx.outputs[i];
        s += &hex::encode(i.value.to_le_bytes());
        s += &get_length_prefixed_string(&i.scriptpubkey);
    }
    s += &hex::encode((tx.locktime as u32).to_le_bytes());
    s += &hex::encode((1 as u32).to_le_bytes()); // end on 4 byte sighash_all flag
    return s
}

// Turns an int into a hex varint string
fn to_hex_var_int(value: u64) -> String {
    return if value < (u8::MAX-2) as u64 {
        hex::encode((value as u8).to_ne_bytes())
    }
    else if value < u16::MAX as u64 {
        let byte_one = hex::encode((u8::MAX-2).to_ne_bytes());
        hex::encode((value as u16).to_le_bytes()) + &byte_one
    }
    else if value < u32::MAX as u64 {
        let byte_one = hex::encode((u8::MAX-1).to_ne_bytes());
        hex::encode((value as u32).to_le_bytes()) + &byte_one
    }
    else { // 64 bit value
        let byte_one = hex::encode((u8::MAX).to_ne_bytes());
        hex::encode((value as u64).to_le_bytes()) + &byte_one
    }
}

// Prefixes a hex string with its length in bytes
pub fn get_length_prefixed_string(s: &str) -> String {
    if s.len() == 0 { return "00".to_string() }
    return to_hex_var_int((s.len()/2) as u64) + s
}