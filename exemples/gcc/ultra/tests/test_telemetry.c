#include <assert.h>
#include <stdio.h>

static double avg(const double *values, int count) {
    double sum = 0.0;
    for (int i = 0; i < count; i++) {
        sum += values[i];
    }
    return count == 0 ? 0.0 : sum / count;
}

int main(void) {
    double readings[] = { 1.0, 2.0, 3.0 };
    double mean = avg(readings, 3);
    assert(mean > 1.9 && mean < 2.1);
    printf("tests ok\n");
    return 0;
}
