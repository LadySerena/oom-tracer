mod Oomkill {
    include!(concat!(env!("OUT_DIR"), "/oomkill.skel.rs"));
}
#[macro_use]
extern crate lazy_static;

use anyhow::bail;
use anyhow::Result;
use libbpf_rs::RingBufferBuilder;
use plain::Plain;
use procfs::process::Process;
use std::time::Duration;
use time::macros::*;
use time::OffsetDateTime;
use Oomkill::*;
unsafe impl Plain for oomkill_bss_types::event {}

// initialize our page size since that's the unit for hiwater_rss. Generally
// one can assume a 4Kb page size but Arm technically supports 16Kb page size
// like on asahi linux...long story
lazy_static! {
    static ref PAGE_SIZE: usize = page_size::get();
}

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

// this function does our mapping. We can't get everything from the kernel so
// while in userspace let's grab some info in procfs.
fn handle_oom_kill(data: &[u8]) -> i32 {
    // this event is the same as our C version but the libbpf crates do some
    // helpful parsing for us which is nice
    let mut event = oomkill_bss_types::event::default();
    // we get a chunk of memory in the form of an array of bytes. We need to tell
    // our program that these bytes are actually our struct
    plain::copy_from_bytes(&mut event, data).expect("data buffer too short");
    // we shouldn't use unwrap in production but in the future we'll have nicer
    // error handling.
    // here we get another struct representing the procfs entry for the given pid
    // that was killed by the oom killer.
    let process_info = Process::new(event.pid).unwrap();
    // grab the commandline and do some ascii to utf8 hackery
    let raw_comm = event.my_comm.clone();
    let temp_comm = String::from_utf8_lossy(&raw_comm);
    let parsed_comm = temp_comm.trim_matches(char::from(0));
    // get the list of cgroups for the process
    let cgroups = process_info.cgroups().unwrap();
    // timestamp
    let now = if let Ok(now) = OffsetDateTime::now_local() {
        let format = format_description!("[hour]:[minute]:[second]");
        now.format(&format)
            .unwrap_or_else(|_| "00:00:00".to_string())
    } else {
        "00:00:00".to_string()
    };

    // I'm not sure if a process can belong to multiple cgroups but we're looking
    // for the path that ends in .scope since that's what docker seems to do
    let cgroup_opt = cgroups
        .iter()
        .find(|&group| group.pathname.contains("scope"));

    // this is more nicer handling, since I prefer to not panic we just set the
    // output to a default value if one isn't present.
    let cgroup_name = match cgroup_opt {
        Some(group) => group.pathname.as_str(),
        None => "no cgroup name",
    };

    // more memory crimes. So our event field hiwater_rss is a 64 bit number but
    // our page size may or not be. So we convert our 64 number into the usize type
    // before doing multiplication.
    let memory_usage = usize::try_from(event.highwater_rss).unwrap() * *PAGE_SIZE;
    // print our output this might be converted to an event or a trace since the
    // cardinality would be pretty intense with the pids and cgroup_names
    println!(
        "{:9} {:<7} {:<7} {:<7} {:<20} {:<10} {:<20}",
        now, event.pid, event.ppid, event.cgroup, memory_usage, parsed_comm, cgroup_name
    );
    // upstream libbpf-rs says this function must return an integer. IDK why but
    // the upstream told the compiler which means we have no choice in the matter.
    0
}

fn main() -> Result<()> {
    // let's build our probe with default options. You can set global variables in
    // your probe file and then set the values via this builder.
    let skel_builder = OomkillSkelBuilder::default();

    // upstream generally bumps the memory limit so I'm copying them
    bump_memlock_rlimit()?;

    // all ebpf programs have this sequency of create builder -> open builder ->
    // loadbuilder -> attach
    let open_skel = skel_builder.open()?;

    let mut skel = open_skel.load()?;
    skel.attach()?;
    // map is the same name as in the bpf.c file
    // we need to connect to our ring buffer that is in kernel space. think of this
    // like a pipe.
    let mut builder = RingBufferBuilder::new();
    // declare our callback function
    builder.add(skel.maps_mut().rb(), handle_oom_kill).unwrap();
    let ring_buffer = builder.build().unwrap();
    println!("beginning to poll our perf buffer");
    // print a nice header for us
    println!(
        "{:9} {:<7} {:<7} {:<7} {:<20} {:<10} {:<20}",
        "timestamp", "pid", "ppid", "cgroup", "highwater memory", "cmdline", "cgroup_name"
    );
    // just poll the buffer for events every 100ms this may not be a production
    // best practice. From there we just enter an infinite loop and process events.
    loop {
        ring_buffer.poll(Duration::from_millis(100))?;
    }
}
