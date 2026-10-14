#include "experiment.h"
/* Lab-scale deterministic bench: no allocation, no libm, no C runtime. */
static AionSim bench_sim;
static void capture(AionBenchPoint *p,const AionSim *s,int step){
 p->step=step;p->time_s=s->sim_time;p->mass_kg=aion_mass(s);
 p->thermal_j=aion_thermal_energy(s);p->mechanical_j=aion_mechanical_energy(s);
 p->intact_bonds=aion_alive_bonds(s);p->fractures=s->fractures;
 p->plastic_events=s->plastic_events;p->plastic_strain=aion_max_plastic_strain(s);
 p->damage=s->max_damage;p->checksum=aion_checksum(s);
}
int aion_bench_run(int material,int protocol,AionBenchResult *out){
 if(!out||material<0||material>=3||protocol<0||protocol>=3)return 0;
 aion_reset(&bench_sim,material);
 out->material=material;out->protocol=protocol;
 capture(&out->points[0],&bench_sim,0);
 for(int step=1;step<=AION_BENCH_STEPS;step++){
  if(protocol!=0 && step%180==0){aion_pull(&bench_sim);}
  if(protocol==2 && step%250==0){aion_heat(&bench_sim);}
  aion_step(&bench_sim,.003);
  if(step%AION_BENCH_INTERVAL==0){
   capture(&out->points[step/AION_BENCH_INTERVAL],&bench_sim,step);
  }
 }
 return 1;
}
/* Explicit decimal serializer with a fixed 6-place precision and 16-char hex
   checksums. It is safe in Win32 freestanding binaries (no sprintf/CRT). */
typedef struct {char *out;unsigned at,cap;int ok;} Writer;
static void putc_(Writer *w,char c){if(w->at+1<w->cap)w->out[w->at++]=c;else w->ok=0;}
static void str(Writer*w,const char*s){while(*s)putc_(w,*s++);}
static void uint_(Writer*w,unsigned long long n){
 char rev[24];unsigned i=0;do{rev[i++]=(char)('0'+n%10);n/=10;}while(n&&i<24);
 while(i)putc_(w,rev[--i]);
}
static void int_(Writer*w,int n){
 if(n<0){putc_(w,'-');uint_(w,(unsigned long long)(-(long long)n));}
 else uint_(w,(unsigned)n);
}
static void fixed_(Writer*w,double v){
 if(!(v>=-1000000000.0&&v<=1000000000.0)){w->ok=0;return;}
 if(v<0){putc_(w,'-');v=-v;}
 unsigned long long scaled=(unsigned long long)(v*1000000.+.5);
 uint_(w,scaled/1000000u);putc_(w,'.');
 unsigned long long fraction=scaled%1000000u;
 unsigned long long div=100000u;
 while(div){putc_(w,(char)('0'+fraction/div));fraction%=div;div/=10u;}
}
static void hex_(Writer*w,unsigned long long n){
 for(int sh=60;sh>=0;sh-=4){unsigned v=(unsigned)((n>>sh)&15);
  putc_(w,(char)(v<10?'0'+v:'a'+v-10));}
}
unsigned aion_bench_csv(const AionBenchResult *results,unsigned count,char*out,unsigned cap){
 if(!results||!out||cap==0||count>AION_BENCH_RUNS)return 0;
 Writer w={out,0,cap,1};
 str(&w,"material,protocol,step,time_s,mass_kg,alive_bonds,fractures,plastic_events,thermal_j,mechanical_j,plastic_strain,max_damage,checksum\r\n");
 for(unsigned r=0;r<count;r++){
  const AionBenchResult *run=&results[r];
  if(run->material<0||run->material>=3||run->protocol<0||run->protocol>=3)return 0;
  for(int i=0;i<AION_BENCH_POINTS;i++){
   const AionBenchPoint *p=&run->points[i];
   int_( &w,run->material);putc_(&w,',');int_(&w,run->protocol);putc_(&w,',');
   int_(&w,p->step);putc_(&w,',');fixed_(&w,p->time_s);putc_(&w,',');
   fixed_(&w,p->mass_kg);putc_(&w,',');int_(&w,p->intact_bonds);putc_(&w,',');
   int_(&w,p->fractures);putc_(&w,',');int_(&w,p->plastic_events);putc_(&w,',');
   fixed_(&w,p->thermal_j);putc_(&w,',');fixed_(&w,p->mechanical_j);putc_(&w,',');
   fixed_(&w,p->plastic_strain);putc_(&w,',');fixed_(&w,p->damage);putc_(&w,',');
   hex_(&w,p->checksum);str(&w,"\r\n");
  }
 }
 if(!w.ok)return 0;
 w.out[w.at]=0;
 return w.at;
}
