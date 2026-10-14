/* AION 0.4: focused plasticity, fatigue and damping checks.
   Uses the same engine as the Windows viewer, no substitute formulas. */
#include "../native/aion_sim.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
static int checks=0;
#define MUST(expr) do { if(!(expr)){fprintf(stderr,"FAIL material_04 line %d: %s\n",__LINE__,#expr);exit(1);}checks++;}while(0)
static AionSim s,t;
static void sample(double dist){
  aion_reset(&s,0);s.node_count=2;s.edge_count=1;s.gravity=0.;s.floor_enabled=0;
  Node *a=&s.nodes[0],*b=&s.nodes[1];
  a->p=(V3){0.,1.,0.}; b->p=(V3){dist,1.,0.};
  a->v=(V3){0.,0.,0.}; b->v=(V3){0.,0.,0.};
  a->m=b->m=1.;a->cv=b->cv=500.;a->r=b->r=.05;
  a->t=b->t=300.;
  Edge *e=&s.edges[0];
  *e=(Edge){.a=0,.b=1,.alive=1,.rest=1.,.k=20.,.limit=5000.,.conductance=0.,
            .original_rest=1.,.yield_strain=.05,.plastic_rate=30.,.damage=0.,
            .damage_onset=.30,.damage_rate=0.,.dashpot=0.};
}
int main(void){
  sample(1.25);
  double initial=aion_total_energy(&s),mass=aion_mass(&s);
  aion_step(&s,.002);
  MUST(s.plastic_events>0);MUST(s.edges[0].rest>1.);
  MUST(aion_max_plastic_strain(&s)>0.);
  MUST(s.plastic_heat>0.);MUST(s.nodes[0].t>300.);
  MUST(fabs(aion_mass(&s)-mass)<1e-12);
  MUST(fabs(aion_total_energy(&s)-initial)<.006); /* Euler discretization */
  double rest=s.edges[0].rest;
  s.edges[0].plastic_rate=0.;aion_step(&s,.002);
  MUST(s.edges[0].rest==rest); /* irreversible */
  sample(1.25);s.edges[0].yield_strain=1.;s.edges[0].plastic_rate=0.;
  s.edges[0].damage_onset=.05;s.edges[0].damage_rate=10.;
  double d0=s.edges[0].damage;
  aion_step(&s,.002);
  MUST(s.edges[0].damage>d0);MUST(s.damage_heat>0.);
  MUST(s.max_damage>0.);MUST(s.plastic_events==0);
  double d1=s.edges[0].damage;
  s.nodes[1].p.x=1.;s.nodes[1].v=(V3){0,0,0};s.nodes[0].v=(V3){0,0,0};
  aion_step(&s,.002);
  MUST(s.edges[0].damage==d1); /* no self healing */
  sample(1.60);s.edges[0].yield_strain=1.;s.edges[0].plastic_rate=0.;
  s.edges[0].damage_onset=.05;s.edges[0].damage_rate=500.;
  for(int n=0;n<15 && s.edges[0].alive;n++){
    s.nodes[0].p.x=0.;s.nodes[1].p.x=1.6;
    s.nodes[0].v=(V3){0,0,0};s.nodes[1].v=(V3){0,0,0};
    aion_step(&s,.002);
  }
  MUST(!s.edges[0].alive);MUST(s.fatigue_fractures==1);MUST(s.fractures==1);
  MUST(s.max_damage>=.999);MUST(aion_alive_bonds(&s)==0);
  sample(1.0);s.edges[0].plastic_rate=0.;s.edges[0].damage_rate=0.;
  s.edges[0].dashpot=2.;s.nodes[0].v.x=-.1;s.nodes[1].v.x=.1;
  aion_step(&s,.002);
  MUST(s.damping_heat>0.);MUST(s.nodes[0].t>300.);
  MUST(s.fractures==0);
  aion_reset(&s,0);aion_reset(&t,0);
  for(int i=0;i<3500;i++){if(i==400){aion_pull(&s);aion_pull(&t);}aion_step(&s,.003);aion_step(&t,.003);}
  MUST(aion_checksum(&s)==aion_checksum(&t));
  MUST(s.plastic_events>0);MUST(s.max_damage>0.);
  MUST(aion_mass(&s)==aion_mass(&t));
  MUST(s.plastic_heat>=0.&&s.damage_heat>=0.&&s.damping_heat>=0.);
  MUST(aion_abi_size()==sizeof(AionSim));
  printf("AION 0.4 material tests: %d assertions PASSED (plasticity/fatigue/damping/replay)\n",checks);
  return 0;
}
