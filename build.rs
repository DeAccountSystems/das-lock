pub use blake2b_rs::{Blake2b, Blake2bBuilder};
use core::str;
use std::{error::Error, process::Command, fs::File, io::{Read, BufWriter, Write}};

trait MustSucceed {
    fn run(&mut self) -> ();
}

impl MustSucceed for Command {
    fn run(&mut self) -> () {
        let output = match self.output() {
            Ok(output) => output,
            Err(_e) => {
                panic!("Failed to execute {:?}", self);
            }
        };
        if !output.status.success() {
            println!(
                "cargo:warning=command: {:?}, err: {}",
                self,
                str::from_utf8(output.stderr.as_slice()).unwrap()
            );
            panic!("{:?}", self)
        }
    }
}

const BUF_SIZE: usize = 8 * 1024;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub fn black2b_hash(file: &mut File) -> [u8; 32] {
    let mut buf = [0u8; BUF_SIZE];
    let mut builder = Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();
    loop {
        let read_bytes = file.read(&mut buf).unwrap();
        if read_bytes > 0 {
            builder.update(&buf[..read_bytes]);
        } else {
            break;
        }
    }
    let mut hash = [0u8; 32];
    builder.finalize(&mut hash);
    hash
}

fn main() -> Result<(), Box<dyn Error>> {
    // Init c deps
    Command::new("sh")
        .arg("-c")
        .arg("git submodule update --init --recursive")
        .run();

    Command::new("sh")
        .current_dir("deps/secp256k1")
        .arg("autogen.sh")
        .run();

    Command::new("./configure")
        .current_dir("deps/secp256k1")
        .args([
            "--with-bignum=no",
            "--with-asm=no",
            "--enable-module-recovery",
            "--enable-endomorphism",
            "--enable-ecmult-static-precomputation",
        ])
        .run();

    Command::new("make").current_dir("deps/secp256k1").run();

    let files = [
        "dispatch",
        "eth_sign.so",
        "ckb_sign.so",
        "tron_sign.so",
        "ed25519_sign.so",
        "ckb_multi_sign.so",
        "doge_sign.so",
    ];

    // Profile = release | debug
    let profile = std::env::var("PROFILE").unwrap();

    let is_release = profile.as_str() == "release";

    let mut build_command = Command::new("make");
    build_command.current_dir(".");

    match is_release {
        true => build_command.arg("all"),
        false => build_command.arg("debug-all"),
    };

    // Overwrite CFLAGS for makefile because newer gcc will introduce warnings in no-array-bounds, no-dangling-pointer, no-stringop-overflow
    build_command.arg("CFLAGS=-Os -fPIC -nostdinc -nostdlib -nostartfiles -fvisibility=hidden -I . -I deps/ckb-c-stdlib -I deps/ckb-c-stdlib/libc -I deps/ckb-c-stdlib/molecule -I deps/secp256k1/src -I deps/secp256k1  -Wall -Werror -Wno-nonnull -Wno-nonnull-compare -Wno-unused-function -Wno-array-bounds -Wno-dangling-pointer -Wno-stringop-overflow");

    build_command.run();

    let out_dir = std::env::var("OUT_DIR").unwrap();

    Command::new("mkdir")
        .args(["-p", &format!("{}/contracts", &out_dir)])
        .run();
    let contracts_path = format!("{}/contracts/mod.rs", &out_dir);
    let contracts_file = File::create(contracts_path).expect("create contracts file failed");
    let mut contracts_buf_writer = BufWriter::new(contracts_file);
    writeln!(&mut contracts_buf_writer, "// Generated by build.rs, do not touch").unwrap();
    for f in files {
        let input_path = format!("build/{}/{}", &profile, f);

        // Copy built files to rust target dir
        Command::new("cp")
            .args([&input_path, format!("{}/contracts/{}", out_dir, f).as_str()])
            .run();
        let hash = {
            let mut file = File::open(&input_path).unwrap();
            black2b_hash(&mut file)
        };

        // Include the binary and hash to rust statically
        write!(
            contracts_buf_writer,
r#"
pub mod {} {{
    pub const HASH: [u8; 32] = {:?};
    pub const BINARY: &[u8] = core::include_bytes!("{}");
}}
"#,
            f.replace(".so", ""),
            hash,
            f
        ).unwrap();
    };

    // Tell cargo when to rebuild
    cargo_emit::rerun_if_changed!(
        "c",
        "src",
        "build.rs"
    );
    Ok(())
}
