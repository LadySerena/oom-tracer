#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

// some parts of the kernel can only be accessed via gpl'ed code
char LICENSE[] SEC("license") = "Dual BSD/GPL";

// wow much global variable
unsigned int my_pid = 0;

// macro to define our bpf program
// see https://www.kernel.org/doc/html/latest/bpf/libbpf/program_types.html
// in this case we're using the tracepoint type
// sudo find /sys/kernel/tracing/events -type d | rg "oom"
// we're gonna trace marking a process to be oom killed
// /sys/kernel/tracing/events/oom/mark_victim
// the trace point is a bit grim isn't it :(
SEC("tp/oom/mark_victim")
int handle_tp(void *ctx) {
  // this is a basic example function
  int pid = bpf_get_current_pid_tgid() >> 32; 
  //pid is in the upper 32 bits and the thread id is in the lower 32 bits so we
  // toss those out just for the pid

  bpf_printk("rust: BPF triggered from PID %d.\n", pid);

  return 0;
  
}