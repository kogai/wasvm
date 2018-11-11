#include <stdio.h>

int subject(int a) {
  if (a == 10) {
    return a + 5;
  }
  if (a != 10) {
    return a + 10;
  }
  return a + 15;
}
