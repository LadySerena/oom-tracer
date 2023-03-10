use libbpf_cargo::SkeletonBuilder;
use std::env;
use std::str;
use std::io::Write;
use std::fs::File;
use std::path::{PathBuf, Path};
use std::process::Command;

// This file tells Cargo that we need non rust code to be
// compiled from source. In our case, bpf code.
// It also will retrieve the vmlinux.h file by invoking bpftools.

const SRC: &str = "src/bpf/oomkill.bpf.c";
const HEADER_SRC: &str = "/sys/kernel/btf/vmlinux";
const HEADER_DEST: &str = "src/bpf/vmlinux.h";

fn main() {
    let mut out =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set in build script"));

    // copy in the vmlinux headers
    let output = Command::new("bpftool")
	.args(&["btf", "dump", "file", HEADER_SRC, "format", "c"])
        .output()
        .expect("Unable to generate vmlinux.h, do you have bpftools installed?");
    generate_file(&HEADER_DEST, &output.stdout);
    println!("cargo:rerun-if-changed={HEADER_SRC}");

    // build the skeleton
    out.push("oomkill.skel.rs");
    SkeletonBuilder::new()
        .source(SRC)
        .build_and_generate(&out)
        .unwrap();
    println!("cargo:rerun-if-changed={SRC}");
}


fn generate_file<P: AsRef<Path>>(path: P, text: &[u8]) {
    let mut f = File::create(path).unwrap();
    f.write_all(text).unwrap()
}
