use ed25519_dalek::{Signer as _, Verifier as _, Signature as DalekSignature, SigningKey, VerifyingKey};
use getrandom::getrandom;
use base64::{engine::general_purpose, Engine as _};

pub type Signature = String;
pub type PublicKey = String;

#[derive(Clone)]
pub struct Keypair {
    pub signing: SigningKey,
    pub verify: VerifyingKey,
}

impl Keypair {
    pub fn generate() -> Self {
        // Seed the signing key from the OS RNG via getrandom to avoid version
        // conflicts between different `rand_core` versions pulled in by deps.
        let mut seed = [0u8; 32];
        getrandom(&mut seed).expect("getrandom");
        let signing = SigningKey::from_bytes(&seed);
        let verify = signing.verifying_key();
        Keypair { signing, verify }
    }

    pub fn public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.verify.as_bytes())
    }
}

pub struct Signer {
    kp: Keypair,
}

pub struct Verifier {
    vk: VerifyingKey,
}

impl Signer {
    pub fn new(kp: Keypair) -> Self { Self { kp } }

    pub fn sign_bytes(&self, bytes: &[u8]) -> String {
        let sig: DalekSignature = self.kp.signing.sign(bytes);
        general_purpose::STANDARD.encode(sig.to_bytes())
    }
}

impl Verifier {
    pub fn from_base64(pk_b64: &str) -> anyhow::Result<Self> {
        let pk = general_purpose::STANDARD.decode(pk_b64)?;
        let vk = VerifyingKey::from_bytes(
            pk.as_slice().try_into().map_err(|_| anyhow::anyhow!("pk length"))?
        )?;
        Ok(Self { vk })
    }

    pub fn verify_bytes(&self, bytes: &[u8], sig_b64: &str) -> anyhow::Result<()> {
        let sb = general_purpose::STANDARD.decode(sig_b64)?;
        let sig = ed25519_dalek::Signature::from_bytes(
            &sb.try_into().map_err(|_| anyhow::anyhow!("sig length"))?
        );
        self.vk.verify(bytes, &sig).map_err(|e| anyhow::anyhow!(e))
    }
}
