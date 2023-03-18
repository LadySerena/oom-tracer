mod Oomkill {
    include!(concat!(env!("OUT_DIR"), "/oomkill.skel.rs"));
}
use anyhow::bail;
use anyhow::Result;
use libbpf_rs::RingBufferBuilder;
use plain::Plain;
use procfs::process::MountInfo;
use procfs::process::Process;
use std::time::Duration;
use time::macros::*;
use time::OffsetDateTime;
use Oomkill::*;
unsafe impl Plain for oomkill_bss_types::event {}

// upstream uses this to bump memory limits for the probes
fn bump_memlock_rlimit() -> Result<()> {
    let rlimit = libc::rlimit {
        rlim_cur: 128 << 20,
        rlim_max: 128 << 20,
    };

    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        bail!("Failed to increase rlimit");
    }

    Ok(())
}

fn handle_oom_kill(data: &[u8]) -> i32 {
    println!("handling oomkill");
    let mut event = oomkill_bss_types::event::default();
    plain::copy_from_bytes(&mut event, data).expect("data buffer too short");
    let process_info = Process::new(event.pid).unwrap();
    let cmdline = process_info.cmdline().unwrap();
    let mounts = process_info.cgroups().unwrap();
    // let mounts = process_info.mountinfo().unwrap_or_else(| | "None".to_string())
    let now = if let Ok(now) = OffsetDateTime::now_local() {
        let format = format_description!("[hour]:[minute]:[second]");
        now.format(&format)
            .unwrap_or_else(|_| "00:00:00".to_string())
    } else {
        "00:00:00".to_string()
    };

    println!(
        "{:8} {:<7} {:<7} {:<7} {:<20} {:?}",
        now, event.pid, event.ppid, event.cgroup, event.highwater_rss, cmdline
    );
    for group in mounts.iter() {
        println!("cgroup: {}", group.pathname)
    }
    0
}

fn handle_lost_oom_kill(cpu: i32, count: u64) {
    eprintln!("lost {count} events on CPU {cpu}");
}

fn main() -> Result<()> {
    println!("Hello, world!");
    let mut skel_builder = OomkillSkelBuilder::default();

    bump_memlock_rlimit()?;

    let mut open_skel = skel_builder.open()?;

    let mut skel = open_skel.load()?;
    skel.attach()?;
    println!("Successfully started! Please run `sudo cat /sys/kernel/debug/tracing/trace_pipe` to see output of the BPF programs.\n");

    // map is the same name as in the bpf.c file
    let mut builder = RingBufferBuilder::new();
    builder.add(skel.maps_mut().rb(), handle_oom_kill).unwrap();
    let ring_buffer = builder.build().unwrap();

    println!("can't get here");

    loop {
        ring_buffer.poll(Duration::from_millis(100))?;
    }
}
