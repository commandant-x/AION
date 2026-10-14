/* Comparisons for the independent Rust candidate: same initial conditions. */
#include <stdio.h>
#include "../native/aion_sim.h"
int main(void){
  printf("mode,steps,seconds,mass,bonds,fractures,contacts,thermal,mechanical,damage,plastic,p0y,p1x\n");
  for(int mode=0;mode<3;mode++){
    AionSim s;aion_reset(&s,mode);aion_pull(&s);aion_heat(&s);
    int targets[]={0,1,5,20,80,400,2000};int passed=0;
    for(unsigned k=0;k<sizeof(targets)/sizeof(targets[0]);k++){
      for(;passed<targets[k];passed++)aion_step(&s,.003);
      printf("%d,%d,%.17g,%.17g,%d,%d,%d,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g\n",
        mode,passed,s.sim_time,aion_mass(&s),aion_alive_bonds(&s),s.fractures,
        s.contacts,aion_thermal_energy(&s),aion_mechanical_energy(&s),
        s.max_damage,aion_max_plastic_strain(&s),s.nodes[0].p.y,s.nodes[1].p.x);
    }
  }
  return 0;
}
