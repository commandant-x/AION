/* Native physics contract checks, all operate on exactly the engine used by Windows GUI. */
#include "../native/aion_sim.h"
#include <stdio.h>
#include <math.h>
#include <stdlib.h>
static int passed=0;
#define CHECK(c) do{if(!(c)){fprintf(stderr,"FAILED: %s @ %d\n",#c,__LINE__);exit(1);}++passed;}while(0)
static int near(double a,double b,double tolerance){return fabs(a-b)<=tolerance;}
static void two(AionSim *s){
 aion_reset(s,0); s->node_count=2;s->edge_count=0;s->fractures=0;s->contacts=0;
 s->gravity=0;s->floor_enabled=0;
 for(int i=0;i<2;i++){s->nodes[i].m=1.;s->nodes[i].r=.2;s->nodes[i].cv=500.;s->nodes[i].t=300.;s->nodes[i].v=(V3){0,0,0};s->nodes[i].component=i;}
}
static AionSim a,b;
int main(void){
 aion_reset(&a,0);
 CHECK(a.node_count==73);CHECK(a.edge_count==404);CHECK(aion_alive_bonds(&a)==404);
 CHECK(near(aion_mass(&a),97.4,1e-8));CHECK(aion_abi_size()==sizeof(AionSim));
 CHECK(aion_checksum(&a)==aion_checksum(&a));
 double mass=aion_mass(&a);int event_count=0;
 for(int i=0;i<10000;i++){aion_step(&a,.003);if(a.fractures>0)event_count=1;
  if(i%300==0){CHECK(isfinite(aion_total_energy(&a)));CHECK(isfinite(aion_mass(&a)));}}
 CHECK(aion_mass(&a)==mass);CHECK(event_count);
 aion_reset(&b,0);for(int i=0;i<10000;i++)aion_step(&b,.003);
 CHECK(aion_checksum(&a)==aion_checksum(&b));
 CHECK(a.fractures==b.fractures);CHECK(a.contacts==b.contacts);
 double heat=aion_thermal_energy(&a);aion_heat(&a);CHECK(aion_thermal_energy(&a)>heat);
 int previous=a.rejected_steps;aion_step(&a,0.);CHECK(a.rejected_steps==previous+1);
 aion_step(&a,INFINITY);CHECK(a.rejected_steps==previous+2);
 aion_reset(&a,1);CHECK(a.nodes[0].kind==1);CHECK(aion_mass(&a)<97.4);
 aion_reset(&a,2);CHECK(a.nodes[0].kind==2);CHECK(aion_mass(&a)>97.4);
 /* Unforced pair: opposite velocities, momentum should be conserved. */
 two(&a);a.nodes[0].p=(V3){-.25,2,0};a.nodes[1].p=(V3){.25,2,0};
 a.nodes[0].v=(V3){1,0,0};a.nodes[1].v=(V3){-1,0,0};
 double e0=aion_total_energy(&a);for(int i=0;i<200;i++)aion_step(&a,.002);
 CHECK(a.contacts>=1);CHECK(near(a.nodes[0].v.x+a.nodes[1].v.x,0,1e-8));
 CHECK(near(aion_total_energy(&a),e0,1e-6));
 /* Bonds keep one body internally connected, avoiding false self-collisions. */
 two(&a);a.nodes[0].p=(V3){0,2,0};a.nodes[1].p=(V3){.1,2,0};
 a.edge_count=1;a.edges[0]=(Edge){.a=0,.b=1,.alive=1,.rest=.1,.k=10,.limit=1000,.conductance=0,.original_rest=.1};
 for(int i=0;i<30;i++)aion_step(&a,.002);
 CHECK(a.contacts==0);CHECK(aion_alive_bonds(&a)==1);
 CHECK(near(a.nodes[1].p.x-a.nodes[0].p.x,.1,1e-8));
 /* Thermal exchange conserves joules and does not invert heat flow. */
 two(&a);a.nodes[0].p=(V3){0,2,0};a.nodes[1].p=(V3){1,2,0};
 a.nodes[0].t=450.;a.nodes[1].t=250.;
 a.edge_count=1;a.edges[0]=(Edge){.a=0,.b=1,.alive=1,.rest=1,.k=10,.limit=1000,.conductance=500,.original_rest=1};
 e0=aion_total_energy(&a);for(int i=0;i<1000;i++)aion_step(&a,.002);
 CHECK(a.nodes[0].t<450&&a.nodes[1].t>250);CHECK(near(aion_total_energy(&a),e0,1e-5));
 /* Fracture converts initial spring strain energy to heat, with no forces. */
 two(&a);a.nodes[0].p=(V3){0,2,0};a.nodes[1].p=(V3){2,2,0};
 a.edge_count=1;a.edges[0]=(Edge){.a=0,.b=1,.alive=1,.rest=1,.k=100,.limit=1,.conductance=0,.original_rest=1};
 e0=aion_total_energy(&a);aion_step(&a,.001);
 CHECK(a.fractures==1);CHECK(aion_alive_bonds(&a)==0);
 CHECK(near(aion_total_energy(&a),e0,1e-6));
 /* Impulse input changes kinetic energy, but not mass. */
 aion_reset(&a,0);mass=aion_mass(&a);double vy=a.nodes[72].v.y;
 aion_impact(&a);CHECK(near(a.nodes[72].v.y,vy-7,1e-12));CHECK(aion_mass(&a)==mass);
 printf("AION 0.4 native shared solver: %d assertions passed; deterministic 10k steps.\n",passed);
 return 0;
}
