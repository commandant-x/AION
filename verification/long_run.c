#define AION_HEADLESS
#include "../windows_viewer/physics.h"
#include <stdio.h>
#include <math.h>
int main(void){
 for(int mode=0;mode<3;mode++){
   material_mode=mode;reset_world();int fail=0;
   for(int t=0;t<80000;t++){
      world_tick(.003);
      for(int i=0;i<node_count;i++) {
        V3 p=nodes[i].p;
        if(!isfinite(p.x)||!isfinite(p.y)||!isfinite(p.z)||fabs(p.x)>100000||fabs(p.y)>100000||fabs(p.z)>100000||!isfinite(nodes[i].t)){fail=t+1;break;}
      }
      if(fail)break;
   }
   printf("material %d: %s fractures=%d contacts=%d time=%.2f\n",mode,fail?"FAIL":"PASS",fractures,contacts,sim_time);
   if(fail)return 1;
 }
 return 0;
}
