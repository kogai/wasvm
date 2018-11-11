#include <stdio.h>

int subject(int a) {
  if (a < 10) {
    return a;
  }
  if (a <= 10) {
    return a + 5;
  }
  if (a > 20) {
    return a + 10;
  }
  if (a >= 20) {
    return a + 15;
  }
  if (a == 30) {
    return a + 20;
  }
  if (a != 40) {
    return a + 25;
  }
  return a + 30;
}
