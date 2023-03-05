mod Oomkill {
    include!(concat!(env!("OUT_DIR"), "/oomkill.skel.rs"));
}
use std::io::Write;
use std::{io::stdout, time::Duration};

use anyhow::Result;
use Oomkill::*;

fn main() -> Result<()> {
    println!("Hello, world!");
    let mut skel_builder = OomkillSkelBuilder::default();
    let mut open_skel = skel_builder.open()?;
    open_skel.bss().my_pid = std::process::id();

    let mut skel = open_skel.load()?;
    skel.attach()?;
    println!("Successfully started! Please run `sudo cat /sys/kernel/debug/tracing/trace_pipe` to see output of the BPF programs.\n");

    let delay = Duration::from_millis(500);
    loop {
        print!(".");
        std::thread::sleep(delay);
        stdout().flush().unwrap();
    }
}
