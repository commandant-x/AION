/* AION 0.4: Windows self-test uses the EXACT same native/aion_sim.c as GUI.
   Define AION_TEST_HOST to also execute these same checks on Linux. */
#include "../native/experiment.h"
#include "physics.h"
#include "../native/checkpoint_portable.h"
#ifdef AION_TEST_HOST
#include <stdio.h>
#include <stdlib.h>
static void logline(const char*s){fputs(s,stdout);}
#define STOP(code) exit(code)
#else
typedef void *HANDLE;
__declspec(dllimport) HANDLE __stdcall GetStdHandle(unsigned int);
__declspec(dllimport) int __stdcall WriteFile(HANDLE,const void *,unsigned int,unsigned int *,void *);
__declspec(dllimport) void __stdcall ExitProcess(unsigned int);
int _fltused = 0;
double sqrt(double x){double y;__asm__ volatile("sqrtsd %1, %0":"=x"(y):"x"(x));return y;}
static void logline(const char*s){unsigned n=0,len=0;while(s[len])len++;WriteFile(GetStdHandle((unsigned int)-11),s,len,&n,0);}
#define STOP(code) ExitProcess(code)
#endif
static unsigned char stored[AION7_MAX_BYTES];
static AionBenchResult bench;
static char bench_csv[AION_BENCH_CSV_CAP];
#define REQUIRE(c) do{if(!(c)){logline("FAIL: " #c "\r\n");STOP(1);}}while(0)
#ifdef AION_TEST_HOST
int main(void)
#else
void mainCRTStartup(void)
#endif
{
 reset_world();double m=total_mass();
 REQUIRE(node_count==73);REQUIRE(edge_count==404);REQUIRE(aion_abi_size()==sizeof(AionSim));
 REQUIRE(m>97.3&&m<97.5);
 for(int i=0;i<3000;i++){
  world_tick(.003);
  for(int p=0;p<node_count;p++){
   REQUIRE(nodes[p].p.x==nodes[p].p.x);REQUIRE(nodes[p].p.y>=nodes[p].r-1e-8);
   REQUIRE(nodes[p].t>0.&&nodes[p].t<100000.);
  }
 }
 REQUIRE(fractures>0);REQUIRE(contacts>0);REQUIRE(m==total_mass());
 unsigned long long signature=aion_checksum(&sim);
 reset_world();for(int i=0;i<3000;i++)world_tick(.003);
 REQUIRE(signature==aion_checksum(&sim));
 double old=total_temperature();heat_sample();REQUIRE(total_temperature()>old);
 material_mode=1;reset_world();REQUIRE(nodes[0].kind==1);
 material_mode=2;reset_world();REQUIRE(nodes[0].kind==2);
 /* Collision between disconnected bodies conserves horizontal momentum. */
 reset_world();node_count=2;edge_count=0;sim.gravity=0;sim.floor_enabled=0;
 nodes[0].p=v3(-.2,2,0);nodes[1].p=v3(.2,2,0);
 nodes[0].r=.2;nodes[1].r=.2;nodes[0].m=1;nodes[1].m=1;
 nodes[0].cv=500;nodes[1].cv=500;
 nodes[0].v=v3(1,0,0);nodes[1].v=v3(-1,0,0);
 world_tick(.003);REQUIRE(contacts>0);
 REQUIRE(nodes[0].v.x+nodes[1].v.x<1e-8 && nodes[0].v.x+nodes[1].v.x> -1e-8);
 /* Bonded overlapping nodes are not driven apart by self-collision. */
 reset_world();node_count=2;edge_count=1;sim.gravity=0;sim.floor_enabled=0;
 nodes[0].p=v3(0,2,0);nodes[1].p=v3(.1,2,0);
 nodes[0].r=.2;nodes[1].r=.2;
 nodes[0].v=v3(0,0,0);nodes[1].v=v3(0,0,0);
 edges[0].a=0;edges[0].b=1;edges[0].rest=.1;edges[0].k=10.;edges[0].limit=1000.;edges[0].conductance=0;edges[0].alive=1;
 world_tick(.003);REQUIRE(nodes[0].p.x==0 && nodes[1].p.x==.1);
 /* New dynamic fields are included in native ABI, and T action injects energy. */
 reset_world();REQUIRE(sim.max_damage==0. && sim.plastic_heat==0.);
 tensile_pull();REQUIRE(nodes[0].v.x<0. && nodes[60].v.x>0.);
 for(int i=0;i<1400;i++)world_tick(.003);
 REQUIRE(sim.max_damage>=0.);REQUIRE(aion_max_plastic_strain(&sim)>=0.);
 REQUIRE(sim.plastic_heat>=0. && sim.damage_heat>=0. && sim.damping_heat>=0.);
 /* Binary checkpoint round-trip and deterministic continuation. */
 reset_world();for(int i=0;i<400;i++)world_tick(.003);
 unsigned long long prior=aion_checksum(&sim);
 unsigned portable_size=aion7_write(&sim,stored,sizeof(stored));REQUIRE(portable_size==aion7_bytes(&sim));
 for(int i=0;i<100;i++)world_tick(.003);
 REQUIRE(aion7_read(&sim,stored,portable_size)==1);
 REQUIRE(aion_checksum(&sim)==prior);
 REQUIRE(aion7_read(&sim,stored,portable_size-1)==0);
 stored[35]^=0x55;REQUIRE(aion7_read(&sim,stored,portable_size)==0);stored[35]^=0x55;
 REQUIRE(aion7_read(&sim,stored,portable_size)==1);
 /* Reference experiment must not mutate the live world. */
 unsigned long long live_hash=aion_checksum(&sim);
 REQUIRE(aion_bench_run(0,1,&bench));
 REQUIRE(bench.points[10].step==1000);
 REQUIRE(aion_bench_csv(&bench,1,bench_csv,sizeof(bench_csv))>1000);
 REQUIRE(aion_checksum(&sim)==live_hash);
 logline("AION CORE 0.9 C REFERENCE SELFTEST: PASS\r\n");
 logline("Replays, energy, mass, contacts, bonded topology, thermal: PASS\r\n");
 STOP(0);
#ifdef AION_TEST_HOST
 return 0;
#endif
}
