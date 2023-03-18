#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <stdint.h>
#include <time.h>
#include <sys/time.h>

int main() {
  pid_t my_pid = getpid();
  pid_t my_ppid = getppid();
  printf("pid: %d\nppid: %d\n", my_pid,my_ppid);
  int multiplier = 256*1024;
  uint64_t counter = 0;
  char *pointer = NULL;

  struct timespec milli;
  milli.tv_sec = 0;
  milli.tv_nsec = 1000*1000;
  while (1) {
    counter++;
    pointer = (char *) malloc(multiplier*1024*1024);
    printf("I've wasted %lu mebibytes\n", counter*multiplier);
    nanosleep(&milli, &milli);
    fork();
  }

  free(pointer);
  return 0;
}
