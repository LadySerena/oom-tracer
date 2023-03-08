#ifndef __OOMKILL_H
#define __OOMKILL_H
// include guard makes sure deps to try to include this twice. makes the
// compiler angry https://en.wikipedia.org/wiki/Include_guard

// TODO grab cgroup info
struct event {
  int pid;
  int ppid;
  long unsigned int highwater_rss;
  int exit_code;
};

#endif /* __OOMKILL_H */