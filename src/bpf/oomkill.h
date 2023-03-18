#ifndef __OOMKILL_H
#define __OOMKILL_H
// include guard makes sure deps to try to include this twice. makes the
// compiler angry https://en.wikipedia.org/wiki/Include_guard

// so C99 by way of this header has standardized integer sizes since int and
// friends are platform dependent vs a uint64_t is 64 unsigned bits no ifs and/
// or buts


// TODO grab cgroup info
// #include <stdint.h>
struct event {
  int pid;
  int ppid;
  uint64_t cgroup;  
  uint64_t highwater_rss;
  int exit_code;
};

#endif /* __OOMKILL_H */