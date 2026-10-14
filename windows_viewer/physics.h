/* AION 0.4: this is ONLY the adapter; the executable runs native/aion_sim.c
   The Rust FFI invokes that exact kernel too. No second C simulation. */
#ifndef AION_PHYSICS_ADAPTER_H
#define AION_PHYSICS_ADAPTER_H
#include "../native/aion_sim.h"
#define SQRT(x) __builtin_sqrt(x)
static AionSim sim;
static double clock_speed=1.;
static int paused=0, material_mode=0;
#define nodes sim.nodes
#define edges sim.edges
#define node_count sim.node_count
#define edge_count sim.edge_count
#define fractures sim.fractures
#define contacts sim.contacts
#define sim_time sim.sim_time
static V3 v3(double x,double y,double z){V3 a={x,y,z};return a;}
static int alive_bonds(void){return aion_alive_bonds(&sim);}
static double total_mass(void){return aion_mass(&sim);}
static double total_temperature(void){return aion_thermal_energy(&sim);}
static void reset_world(void){aion_reset(&sim,material_mode);}
static void world_tick(double dt){aion_step(&sim,dt);}
static void impulse(void){aion_impact(&sim);}
static void tensile_pull(void){aion_pull(&sim);}
static void heat_sample(void){aion_heat(&sim);}
#endif
