use secp256k1::{SecretKey, PublicKey, Secp256k1, Message};
use secp256k1::All;

//use secp256k1::hashes::sha256; // Not sure how to double hash with this one, so using something else
use sha2::{Sha256, Digest};

pub struct ECC {
    curve: Secp256k1<All>, sk: SecretKey, pk: PublicKey
}

impl ECC {
    pub fn new(sk_string: &str) -> ECC {
        let curve = Secp256k1::new();
        let sk_vec = hex::decode(sk_string).unwrap();
        let sk = SecretKey::from_slice(&sk_vec).unwrap();
        let pk = PublicKey::from_secret_key(&curve, &sk);
        return ECC { curve, sk, pk }
    }

    pub fn get_pk_string(&self) -> String {
        return hex::encode(self.pk.serialize())
    }

    pub fn sign_ecdsa_der(&self, hex_string: &str) -> String {
        let data = Sha256::digest(Sha256::digest(hex::decode(hex_string).unwrap()));
        let message = Message::from_slice(&data).unwrap();
        let sig = self.curve.sign_ecdsa(&message, &self.sk);
        let sig_string = hex::encode(sig.serialize_der());
        return sig_string
    }
}