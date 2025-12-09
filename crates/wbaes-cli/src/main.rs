//! Command-line interface for `whitebox-aes-rs`.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use aes_core::{decrypt_block, encrypt_block, expand_key, Aes128Key};
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use wbaes_gen::{Generator, GeneratorConfig, WbInstance256};
use wbaes_runtime::WbCipher256;

/// White-box AES CLI.
#[derive(Parser)]
#[command(
    name = "wbaes",
    version,
    author,
    about = "White-box AES CLI (Baek–Cheon–Hong revisited)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a white-box instance from a key.
    Gen {
        /// AES-128 key as 32 hex characters.
        #[arg(long, value_name = "HEX")]
        key_hex: String,
        /// Output path for the serialized instance.
        #[arg(long, value_name = "FILE")]
        out: PathBuf,
        /// Optional RNG seed for reproducible generation.
        #[arg(long)]
        seed: Option<u64>,
        /// Enable external encodings (defaults off for easier testing).
        #[arg(long, default_value_t = false)]
        external_encodings: bool,
    },
    /// Encrypt 32-byte blocks from a file using a white-box instance.
    Enc {
        /// Path to the serialized instance.
        #[arg(long, value_name = "FILE")]
        instance: PathBuf,
        /// Input file (must be a multiple of 32 bytes).
        #[arg(long, value_name = "FILE")]
        input: PathBuf,
        /// Output ciphertext path.
        #[arg(long, value_name = "FILE")]
        output: PathBuf,
    },
    /// Decrypt 32-byte blocks using the AES key (assumes no external encodings).
    Dec {
        /// Path to the serialized instance (used to sanity-check encoding settings).
        #[arg(long, value_name = "FILE")]
        instance: PathBuf,
        /// AES-128 key as 32 hex characters.
        #[arg(long, value_name = "HEX")]
        key_hex: String,
        /// Input file (ciphertext).
        #[arg(long, value_name = "FILE")]
        input: PathBuf,
        /// Output plaintext path.
        #[arg(long, value_name = "FILE")]
        output: PathBuf,
    },
    /// Verify a white-box instance matches AES for random samples.
    Check {
        /// Path to the serialized instance.
        #[arg(long, value_name = "FILE")]
        instance: PathBuf,
        /// AES-128 key as 32 hex characters.
        #[arg(long, value_name = "HEX")]
        key_hex: String,
        /// Number of random samples to test.
        #[arg(long, default_value_t = 4)]
        samples: usize,
        /// Optional RNG seed for reproducibility.
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Run a local demo: generate key + instance, encrypt random data, decrypt back.
    Demo {
        /// Optional RNG seed for reproducibility.
        #[arg(long)]
        seed: Option<u64>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Gen {
            key_hex,
            out,
            seed,
            external_encodings,
        } => cmd_gen(&key_hex, &out, seed, external_encodings),
        Commands::Enc {
            instance,
            input,
            output,
        } => cmd_enc(&instance, &input, &output),
        Commands::Dec {
            instance,
            key_hex,
            input,
            output,
        } => cmd_dec(&instance, &key_hex, &input, &output),
        Commands::Check {
            instance,
            key_hex,
            samples,
            seed,
        } => cmd_check(&instance, &key_hex, samples, seed),
        Commands::Demo { seed } => cmd_demo(seed),
    }
}

fn cmd_gen(
    key_hex: &str,
    out: &PathBuf,
    seed: Option<u64>,
    external_encodings: bool,
) -> Result<()> {
    let key = parse_key_hex(key_hex)?;
    let rng = seeded_rng(seed);
    let mut gen = Generator::with_config(rng, GeneratorConfig { external_encodings });
    let instance = gen.generate_instance(&key);
    let bytes = instance.to_bytes().context("serialize instance")?;
    fs::write(out, bytes).with_context(|| format!("write {}", out.display()))?;
    Ok(())
}

fn cmd_enc(instance_path: &PathBuf, input_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    let instance = load_instance(instance_path)?;
    let cipher = WbCipher256::new(instance);
    let mut data =
        fs::read(input_path).with_context(|| format!("read {}", input_path.display()))?;
    if data.len() % 32 != 0 {
        bail!("input length must be a multiple of 32 bytes");
    }
    for chunk in data.chunks_mut(32) {
        let mut block = [0u8; 32];
        block.copy_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        chunk.copy_from_slice(&block);
    }
    fs::write(output_path, data).with_context(|| format!("write {}", output_path.display()))?;
    Ok(())
}

fn cmd_dec(
    instance_path: &PathBuf,
    key_hex: &str,
    input_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<()> {
    let instance = load_instance(instance_path)?;
    if instance.encodings.output.is_some() {
        bail!("decryption is not supported when an external output encoding is present");
    }
    let key = parse_key_hex(key_hex)?;
    let round_keys = expand_key(&key);
    let mut data =
        fs::read(input_path).with_context(|| format!("read {}", input_path.display()))?;
    if data.len() % 32 != 0 {
        bail!("input length must be a multiple of 32 bytes");
    }
    for chunk in data.chunks_mut(32) {
        let mut b1 = [0u8; 16];
        let mut b2 = [0u8; 16];
        b1.copy_from_slice(&chunk[..16]);
        b2.copy_from_slice(&chunk[16..]);
        let pt1 = decrypt_block(&b1, &round_keys);
        let pt2 = decrypt_block(&b2, &round_keys);
        chunk[..16].copy_from_slice(&pt1);
        chunk[16..].copy_from_slice(&pt2);
    }
    fs::write(output_path, data).with_context(|| format!("write {}", output_path.display()))?;
    Ok(())
}

fn cmd_check(
    instance_path: &PathBuf,
    key_hex: &str,
    samples: usize,
    seed: Option<u64>,
) -> Result<()> {
    let instance = load_instance(instance_path)?;
    if instance.encodings.output.is_some() {
        bail!("check expects instances with output encodings folded into the tables");
    }
    let cipher = WbCipher256::new(instance);
    let key = parse_key_hex(key_hex)?;
    let round_keys = expand_key(&key);
    let mut rng = seeded_rng(seed);

    for _ in 0..samples {
        let mut block = [0u8; 32];
        rng.fill_bytes(&mut block);
        let mut expected = block;
        let mut first = [0u8; 16];
        let mut second = [0u8; 16];
        first.copy_from_slice(&block[..16]);
        second.copy_from_slice(&block[16..]);
        let b1 = encrypt_block(&first, &round_keys);
        let b2 = encrypt_block(&second, &round_keys);
        expected[..16].copy_from_slice(&b1);
        expected[16..].copy_from_slice(&b2);

        let mut actual = block;
        cipher.encrypt_block(&mut actual);
        if actual != expected {
            bail!("mismatch between white-box and AES outputs");
        }
    }
    Ok(())
}

fn cmd_demo(seed: Option<u64>) -> Result<()> {
    let mut rng = seeded_rng(seed);
    let mut key_bytes = [0u8; 16];
    rng.fill_bytes(&mut key_bytes);
    let key = Aes128Key::from(key_bytes);

    let gen_seed = derive_seed(&mut rng);
    let mut gen = Generator::with_config(
        ChaCha20Rng::from_seed(gen_seed),
        GeneratorConfig {
            external_encodings: false,
        },
    );
    let instance = gen.generate_instance(&key);
    let cipher = WbCipher256::new(instance);

    let mut block = [0u8; 32];
    rng.fill_bytes(&mut block);
    let plaintext_hex = hex::encode(block);

    let round_keys = expand_key(&key);
    cipher.encrypt_block(&mut block);
    let ciphertext_hex = hex::encode(block);

    let mut decrypted = block;
    let mut first = [0u8; 16];
    let mut second = [0u8; 16];
    first.copy_from_slice(&decrypted[..16]);
    second.copy_from_slice(&decrypted[16..]);
    let pt1 = decrypt_block(&first, &round_keys);
    let pt2 = decrypt_block(&second, &round_keys);
    decrypted[..16].copy_from_slice(&pt1);
    decrypted[16..].copy_from_slice(&pt2);

    let decrypted_hex = hex::encode(decrypted);
    println!("demo key: {}", hex::encode(key_bytes));
    println!("plaintext: {}", plaintext_hex);
    println!("ciphertext: {}", ciphertext_hex);
    println!("decrypted: {}", decrypted_hex);
    if decrypted_hex != plaintext_hex {
        bail!("demo roundtrip failed");
    }
    Ok(())
}

fn parse_key_hex(hex_str: &str) -> Result<Aes128Key> {
    let bytes = hex::decode(hex_str.trim()).context("decode key hex")?;
    if bytes.len() != 16 {
        bail!("AES-128 key must be 16 bytes (32 hex characters)");
    }
    let mut key = [0u8; 16];
    key.copy_from_slice(&bytes);
    Ok(Aes128Key::from(key))
}

fn load_instance(path: &PathBuf) -> Result<WbInstance256> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    WbInstance256::from_bytes(&bytes).context("deserialize instance")
}

fn seeded_rng(seed: Option<u64>) -> impl RngCore + CryptoRng {
    match seed {
        Some(value) => {
            let mut seed_bytes = [0u8; 32];
            seed_bytes[..8].copy_from_slice(&value.to_le_bytes());
            ChaCha20Rng::from_seed(seed_bytes)
        }
        None => {
            let mut seed_bytes = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut seed_bytes);
            ChaCha20Rng::from_seed(seed_bytes)
        }
    }
}

fn derive_seed(rng: &mut impl RngCore) -> [u8; 32] {
    let mut seed_bytes = [0u8; 32];
    rng.fill_bytes(&mut seed_bytes);
    seed_bytes
}
