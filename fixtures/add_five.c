#include <stdio.h>

int add_five(int a) {
  return a + 5;
}

int subject(int a, int b) {
  return add_five(a) + add_five(b);
}
