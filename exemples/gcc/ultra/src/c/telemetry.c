#include <stdio.h>
#include <string.h>

static double avg(const double *values, int count) {
    double sum = 0.0;
    for (int i = 0; i < count; i++) {
        sum += values[i];
    }
    return count == 0 ? 0.0 : sum / count;
}

int main(void) {
    const char *sensor = "alpha-7";
    double readings[] = { 12.4, 11.9, 12.1, 12.8, 11.7 };
    int n = (int)(sizeof(readings) / sizeof(readings[0]));
    double mean = avg(readings, n);

    printf("telemetry_c\n");
    printf("sensor: %s\n", sensor);
    printf("samples: %d\n", n);
    printf("avg: %.2f\n", mean);
    printf("checksum: %zu\n", strlen(sensor) + (size_t)n);
    return 0;
}
