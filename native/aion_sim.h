#ifndef AION_SIM_04_H
#define AION_SIM_04_H
/* AION 0.4 — one portable, deterministic C99 experimental physics kernel.
   SI model (coarse-grained sample, NOT atomistic physics): m, kg, s, K, J.
   AION is NOT an atomistic model, nor a numerically validated continuum solver.
   ABI is shared between native Windows viewer, Linux tests and Rust FFI.
   No game-specific item names, no recipes, no hidden progression tree. */
#ifdef __cplusplus
extern "C" {
#endif
#define AION_MAX_NODES 128
#define AION_MAX_BONDS 1024
#define AION_G 9.81
typedef struct { double x, y, z; } V3;
typedef struct { V3 p, v; double r, m, t, cv; int kind, component; } Node;
typedef struct { int a, b, alive; double rest, k, limit, conductance;
  double original_rest, yield_strain, plastic_rate, damage, damage_onset, damage_rate, dashpot; } Edge;
typedef struct {
  Node nodes[AION_MAX_NODES];
  Edge edges[AION_MAX_BONDS];
  int node_count, edge_count, fractures, contacts, material_mode, rejected_steps;
  int plastic_events, fatigue_fractures;
  double sim_time, gravity, max_strain, max_contact_impulse;
  double plastic_heat, damage_heat, damping_heat, max_damage;
  int floor_enabled;
} AionSim;
void aion_reset(AionSim *s, int material_mode);
void aion_step(AionSim *s, double dt);
void aion_impact(AionSim *s);
void aion_heat(AionSim *s);
void aion_pull(AionSim *s);
double aion_max_plastic_strain(const AionSim *s);
int aion_alive_bonds(const AionSim *s);
double aion_mass(const AionSim *s);
double aion_thermal_energy(const AionSim *s);
double aion_mechanical_energy(const AionSim *s);
double aion_total_energy(const AionSim *s);
unsigned long long aion_checksum(const AionSim *s);
/* Experimental checkpoint format 0.6: ABI-bound state with version, magic and FNV-1a integrity.
   Only load files from the same build/architecture. This is NOT a durable portable format.
   Decoding validates node and edge bounds before changing destination state.
*/
unsigned int aion_checkpoint_bytes(void);
unsigned int aion_checkpoint_write(const AionSim *s,unsigned char *output,unsigned int capacity);
int aion_checkpoint_read(AionSim *dst,const unsigned char *bytes,unsigned int length);
/* Introspection for foreign-language ABI compatibility */
unsigned int aion_abi_size(void);
#ifdef __cplusplus
}
#endif
#endif
