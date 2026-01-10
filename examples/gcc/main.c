// src/main.c
#include <stdio.h>

static int add(int a, int b) {
    return a + b;
}

int main(void) {
    int x = 40;
    int y = 2;
    printf("muffin: %d + %d = %d\n", x, y, add(x, y));
    return 0;
}