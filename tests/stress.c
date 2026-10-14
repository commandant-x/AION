/* Extended simulation test across three material profiles and repeated impacts. */
#include "../native/aion_sim.h"
#include <math.h>
#include <stdio.h>
static AionSim s;
int main(void){
 for(int profile=0;profile<3;profile++){
  aion_reset(&s,profile);
  double mass=aion_mass(&s);
  for(int i=0;i<15000;i++){
    if(i%901==0)aion_impact(&s);
    if(i%1201==0)aion_heat(&s);
    aion_step(&s,.003);
    if(i%200==0){
      if(!isfinite(aion_total_energy(&s)) || aion_mass(&s)!=mass) return 20+profile;
      for(int p=0;p<s.node_count;p++){
        if(!isfinite(s.nodes[p].p.x)||!isfinite(s.nodes[p].p.y)||!isfinite(s.nodes[p].p.z)||
           !isfinite(s.nodes[p].t)||s.nodes[p].t<0) return 30+profile;
      }
    }
  }
  printf("Profile %d: 15000 steps PASS / fracture=%d collision=%d\n",profile,s.fractures,s.contacts);
 }
 return 0;
}
