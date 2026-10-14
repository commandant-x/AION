/* Supplemental platform-independent C checks for the native reference solver. */
#define AION_HEADLESS
#include "../windows_viewer/physics.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>

static int nearly(double a,double b,double tol){return fabs(a-b)<tol;}
static void clear(void){node_count=0;edge_count=0;fractures=0;contacts=0;sim_time=0.;}
static void thermal_exchange(void){
 clear();int a=add_node(0.,100.,0.,.08,2.,360.,0),b=add_node(1.,100.,0.,.08,2.,260.,0);
 add_edge(a,b,30.,1e5);
 double energy_before=total_temperature();
 for(int i=0;i<1000;i++)world_tick(.002);
 assert(nearly(energy_before,total_temperature(),1e-5));
 assert(nodes[a].t<360. && nodes[b].t>260.);
 puts("[PASS] Isolated heat exchange conserves thermal energy");
}
static void momentum_and_thermal_contact(void){
 clear();int a=add_node(-.085,3.,0.,.1,3.,290.,0),b=add_node(.085,3.,0.,.1,5.,290.,0);
 nodes[a].v.x=1.;nodes[b].v.x=-1.;
 double p0=nodes[a].m*nodes[a].v.x+nodes[b].m*nodes[b].v.x;
 double ke0=.5*nodes[a].m*nodes[a].v.x*nodes[a].v.x+.5*nodes[b].m*nodes[b].v.x*nodes[b].v.x;
 double h0=total_temperature();
 world_tick(.001);
 double p1=nodes[a].m*nodes[a].v.x+nodes[b].m*nodes[b].v.x;
 double ke1=.5*nodes[a].m*nodes[a].v.x*nodes[a].v.x+.5*nodes[b].m*nodes[b].v.x*nodes[b].v.x;
 assert(contacts==1);assert(nearly(p0,p1,1e-9));assert(ke1<ke0);
 assert(nearly(ke0+h0,ke1+total_temperature(),1e-6));
 puts("[PASS] Contact impulse conserves horizontal momentum and kinetic+thermal energy");
}
static void fracture_energy(void){
 clear();int a=add_node(-.5,3.,0.,.1,2.,290.,0),b=add_node(.5,3.,0.,.1,2.,290.,0);
 add_edge(a,b,500.,10.);nodes[b].p.x+=.3;
 double h0=total_temperature();
 world_tick(.001);
 assert(fractures==1 && !edges[0].alive);
 assert(nearly(total_temperature()-h0,.5*500*.3*.3,1e-6));
 puts("[PASS] Broken spring transfers stored strain to heat");
}
static void deterministic_replay(void){
 material_mode=0;reset_world();for(int i=0;i<2000;i++)world_tick(.003);
 double x=nodes[5].p.x,y=nodes[40].p.y,h=total_temperature(),m=total_mass();int f=fractures,c=contacts;
 reset_world();for(int i=0;i<2000;i++)world_tick(.003);
 assert(x==nodes[5].p.x && y==nodes[40].p.y && h==total_temperature());
 assert(m==total_mass() && f==fractures && c==contacts);
 puts("[PASS] Deterministic replay over 2,000 physics steps");
}
int main(void){thermal_exchange();momentum_and_thermal_contact();fracture_energy();deterministic_replay();puts("FOUR SUPPLEMENTARY PHYSICS TESTS: PASS");return 0;}
