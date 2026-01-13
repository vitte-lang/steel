#include <iostream>
#include <numeric>
#include <string>
#include <vector>

struct Telemetry {
    std::string sensor;
    std::vector<double> readings;
};

static double avg(const std::vector<double>& values) {
    if (values.empty()) {
        return 0.0;
    }
    double sum = std::accumulate(values.begin(), values.end(), 0.0);
    return sum / static_cast<double>(values.size());
}

int main() {
    Telemetry t{ "beta-3", { 9.8, 10.1, 9.9, 10.4, 10.0 } };
    double mean = avg(t.readings);

    std::cout << "telemetry_cpp" << '\n';
    std::cout << "sensor: " << t.sensor << '\n';
    std::cout << "samples: " << t.readings.size() << '\n';
    std::cout << "avg: " << mean << '\n';
    std::cout << "checksum: " << (t.sensor.size() + t.readings.size()) << '\n';
    return 0;
}
