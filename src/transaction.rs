#[derive(Debug)]
pub struct Content {
    pub txid: String, pub version: u64, pub locktime: u64, pub inputs: Vec<Input>, pub outputs: Vec<Output>
}

impl Content {
    // Checks if two transactions are the same (useful for debug to compare with Core)
    pub fn compare(&self, transaction: &Content) -> bool {
        let (a, b) = (self, transaction);
        if a.txid       != b.txid ||
        a.version       != b.version ||
        a.locktime      != b.locktime ||
        a.inputs.len()  != b.inputs.len() || 
        a.outputs.len() != b.outputs.len() { 
            return false
        }
        for i in 0..a.inputs.len() {
            let (a, b) = (&a.inputs[i], &b.inputs[i]);
            if a.txid   != b.txid || 
            a.vout      != b.vout ||
            a.scriptsig != b.scriptsig ||
            a.sequence  != b.sequence {
                return false
            }
        }
        for i in 0..self.outputs.len() {
            let (a, b) = (&a.outputs[i], &b.outputs[i]);
            if a.value      != b.value ||
            a.scriptpubkey  != b.scriptpubkey {
                return false
            }
        }
        return true
    }
}

#[derive(Debug, Clone)]
pub struct Output {
    pub value: u64, pub scriptpubkey: String
}

#[derive(Debug, Clone)]
pub struct Input {
    pub txid: String, pub vout: u64, pub scriptsig: String, pub sequence: u64
}