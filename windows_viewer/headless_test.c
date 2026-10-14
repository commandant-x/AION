#define AION_HEADLESS
#include "physics.h"
#include <stdio.h>
#include <assert.h>
int main(void){
 reset_world();const double mass=total_mass();const double initial_thermal=total_temperature();
 assert(node_count==73 && edge_count>100);
 for(int i=0;i<2000;i++){
   world_tick(.003);
   for(int k=0;k<node_count;k++){
     assert(isfinite(nodes[k].p.x) && isfinite(nodes[k].p.y) && isfinite(nodes[k].p.z));
     assert(isfinite(nodes[k].t) && nodes[k].t>=0. && nodes[k].p.y>=nodes[k].r-1e-7);
   }
 }
 assert(mass==total_mass());
 printf("AION 0.2 physics stress test: PASS\nparticles=%d bonds=%d remaining=%d fractures=%d contacts=%d\ninitial_thermal=%.2f J, final_thermal=%.2f J, mass=%.4f kg, time=%.2f s\n",node_count,edge_count,alive_bonds(),fractures,contacts,initial_thermal,total_temperature(),total_mass(),sim_time);
 reset_world();double t0=total_temperature();heat_sample();assert(total_temperature()>t0);
 puts("heat injection: PASS");
 return 0;
}
