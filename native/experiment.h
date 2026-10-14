#ifndef AION_EXPERIMENT_08_H
#define AION_EXPERIMENT_08_H
#include "aion_sim.h"
/* Explicit comparable simulations. The C kernel is reference only; no Rust
   compilation is claimed. A single fixed dt and protocol are enforced. */
#define AION_BENCH_STEPS 1000
#define AION_BENCH_INTERVAL 100
#define AION_BENCH_POINTS (AION_BENCH_STEPS / AION_BENCH_INTERVAL + 1)
#define AION_BENCH_MATERIALS 3
#define AION_BENCH_PROTOCOLS 3
#define AION_BENCH_RUNS (AION_BENCH_MATERIALS*AION_BENCH_PROTOCOLS)
#define AION_BENCH_CSV_CAP 56000u

typedef struct {
 int step, intact_bonds, fractures, plastic_events;
 double time_s, mass_kg, thermal_j, mechanical_j, plastic_strain, damage;
 unsigned long long checksum;
} AionBenchPoint;
typedef struct {
 int material, protocol;
 AionBenchPoint points[AION_BENCH_POINTS];
} AionBenchResult;
/* Protocol 0: unforced; 1: cyclic tension; 2: tension + controlled heating. */
int aion_bench_run(int material, int protocol, AionBenchResult *out);
/* Pure deterministic serializer: 0 on invalid data / insufficient capacity.
   Returns exact written byte count excluding the terminal NUL. */
unsigned aion_bench_csv(const AionBenchResult *results, unsigned count,
                       char *out, unsigned cap);
#endif
