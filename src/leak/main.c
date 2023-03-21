#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <stdint.h>

int main() {
  int multiplier = 256*1024;

  while (1) {
    malloc(multiplier*1024*1024);
  }

  return 0;
}
