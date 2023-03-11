# BCC vs libbpf

The tl;dr is that BPF programs (which leverage k(ret)probes) that are compiled using BCC is not portable, while those built with libbpf / CO-RE are.

## kprobes

In our example, we are specifically looking at the use of kernel probes from within the BPF vm (yes, it's a virtual machine insside the kernel with its own instruction set, etc).

When a kprobe is placed on an instruction in kernel space, it saves the target instruction, and replaces it with a breakpoint instruction.

This then causes the CPU to trap and call the kprobe handler code.

This is where BPF comes in, it will attach itself to the kernel's code path and do useful things like, save some data from a kernel struct into
a BPF map (TODO add links). This is beneficial since it all occurs in the kernel space, no context switching required!

Afterwards, the kprobe post_handler will hand back control to the kernel, and that's that!


## A kprobe use case

A contrived example:

A program could be written to track any OOM kills that occur on a system. Using kprobes we could intecept the call to oom_kill_handler and retrive information about the process that
was killed and write it to kubernetes config map for easy consumption.


## The problem with BCC

It should be apparent that the program requires knowledge of kernel data structures. A BCC compiled bpf program from one linux version may be completely
broken if ran on another version. Why? The structs and memory layout may have changed! Any offsets the program relies on to pull out data may be pointing to garbage!


Another issue is that BCC does a _lot_ of hidden work to provide a better developer experience. Some structs (in the network stack especially) are represented 
in the kernel using fields that are different from what the API exposes. BCC has to perform all of these translations and renames in code. This results in a larger binary that
depends on the linux headers being present. What if we are running a specially compiled kernel and want to run the program on another kernel which doesn't have any headers present?
It's impossible. We cannot.

If we wanted to simply count the amount of times a kernel function was called, we don't need to worry about offsets. But such trivial applications are not as useful!

## How does libbpf / CO-RE help?

libbpf / CO-RE, in short, makes it easier to write portable BPF applications.

The RE of CO-RE means RElocatable. bpftools can generate a headers file (vmlinux.h) which contains all the BTF definitions for a given kernel. This is portable, plus
it exposes types which are not normally exposed by the linux-headers.

Since libbpf can use the BTF definitions inside of vmlinux.h, the resulting BPF byte code (which is generated, validated and run at run-time) is simpler, and tailored to 
the kernel it is being run on.

Libpf is also embedded into the application, there is no requirement of bcc / libbpf being installed on the target machine.
```
 % ldd target/release/oom-tracer                                                                                                          :)
        linux-vdso.so.1 (0x00007ffd65495000)
        libelf.so.1 => /usr/lib/libelf.so.1 (0x00007f46adc2d000)
        libz.so.1 => /usr/lib/libz.so.1 (0x00007f46adc13000)
        libgcc_s.so.1 => /usr/lib/libgcc_s.so.1 (0x00007f46adbf3000)
        libc.so.6 => /usr/lib/libc.so.6 (0x00007f46ada0c000)
        /lib64/ld-linux-x86-64.so.2 => /usr/lib64/ld-linux-x86-64.so.2 (0x00007f46addd9000)
```

For simple programs, it probably won't make a huge difference than using BCC, but I think the pay off comes from more complex applications. It allows you
to focus on your goal, instead of worrying if a field is now at offset 24 instead of 16.


## Why Compute is interested in leveraging BPF

In Compute, we are extremely interested in observability. From monitoring the number of pods running in a cluster, to the number of retransmitted TCP segments in a given node, we are always observing
the state of the platform.

This talk came from the fact we don't have good metrics around networking. Specifically at the pod level. While we can easily tell (as mentioned above) the number of TCP retransmits occuring on a node,
we can't tell which pod is affected.

We have a monitor called `node_network_bad` (TODO: confirm name) which, when it fires, the node is drained and the problem, though unknown, goes away.

Trying to retroactively infer what went wrong is a crapshoot, the amount of noise our clusters generate is huge (10k logs per minute, TODO confirm), and our metrics often don't reveal anything interesting.
What doesn't help is when a node is drained, there is often an uptick in network related errors (as connections are closing, being refused etc). It's an ocean of red herrings!

Imagine if we could introspect the number of TCP SYNs vs a count of TCP SYN-ACKs per pod. It'd give us the tools to only drain a node if all pods are affected, or to
evict a single pod instead.

We think that libbpf will give us the portability to run these tools in our clusters as daemonsets. Without spending too much time ensuring that each kernel version change won't break anything.

