#ifndef __OOMKILL_H
#define __OOMKILL_H
// include guard makes sure deps to try to include this twice. makes the
// compiler angry https://en.wikipedia.org/wiki/Include_guard

#define PROBE_COMM_LEN 16

struct event {
  int pid;
  int ppid;
  unsigned char my_comm[PROBE_COMM_LEN];
  unsigned long long cgroup;  
  unsigned long long highwater_rss;
  int exit_code;
};

#endif /* __OOMKILL_H */