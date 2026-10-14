#include "../native/experiment.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
static AionBenchResult runs[AION_BENCH_RUNS],again;
static char csv[AION_BENCH_CSV_CAP],other[AION_BENCH_CSV_CAP];
static int checks=0;
#define CHECK(expr) do{if(!(expr)){fprintf(stderr,"FAIL line %d: %s\n",__LINE__,#expr);return 1;}checks++;}while(0)
int main(void){
 CHECK(!aion_bench_run(-1,0,&again));
 CHECK(!aion_bench_run(3,0,&again));
 CHECK(!aion_bench_run(0,-1,&again));
 CHECK(!aion_bench_run(0,3,&again));
 CHECK(!aion_bench_run(0,0,0));
 for(int m=0;m<3;m++)for(int p=0;p<3;p++){
  AionBenchResult *r=&runs[m*3+p];
  CHECK(aion_bench_run(m,p,r));
  CHECK(r->points[0].step==0 && r->points[0].fractures==0);
  double mass=r->points[0].mass_kg;
  int last_fracture=0;
  for(int k=0;k<AION_BENCH_POINTS;k++){
   const AionBenchPoint *x=&r->points[k];
   CHECK(x->step==k*100);
   CHECK(x->time_s>=k*.299999 && x->time_s<=k*.300001);
   CHECK(x->mass_kg==mass);
   CHECK(x->fractures>=last_fracture);
   CHECK(x->intact_bonds+x->fractures==404);
   CHECK(x->thermal_j>0 && x->mechanical_j==x->mechanical_j);
   CHECK(x->plastic_strain>=0 && x->damage>=0 && x->damage<=1);
   last_fracture=x->fractures;
  }
  CHECK(aion_bench_run(m,p,&again));
  for(int k=0;k<AION_BENCH_POINTS;k++) CHECK(r->points[k].checksum==again.points[k].checksum);
 }
 CHECK(runs[1].points[10].checksum!=runs[0].points[10].checksum);
 CHECK(runs[2].points[10].checksum!=runs[1].points[10].checksum);
 CHECK(runs[4].points[10].checksum!=runs[3].points[10].checksum);
 CHECK(runs[7].points[10].checksum!=runs[6].points[10].checksum);
 CHECK(runs[2].points[10].thermal_j>runs[1].points[10].thermal_j);
 unsigned n=aion_bench_csv(runs,AION_BENCH_RUNS,csv,sizeof(csv));
 CHECK(n>4000 && n<sizeof(csv));
 CHECK(strstr(csv,"material,protocol,step") == csv);
 CHECK(strstr(csv,"2,2,1000,")!=NULL);
 CHECK(!aion_bench_csv(runs,AION_BENCH_RUNS,other,n));
 CHECK(!aion_bench_csv(runs,AION_BENCH_RUNS,other,0));
 CHECK(!aion_bench_csv(runs,AION_BENCH_RUNS+1,other,sizeof(other)));
 CHECK(aion_bench_csv(runs,AION_BENCH_RUNS,other,sizeof(other))==n);
 CHECK(memcmp(csv,other,n+1)==0);
 FILE *f=fopen("experiments/AION_CORE_0_8_reference.csv","wb");
 CHECK(f!=NULL);CHECK(fwrite(csv,1,n,f)==n);CHECK(fclose(f)==0);
 printf("AION CORE 0.8 experiment runner: %d checks PASS, %u CSV bytes, 9 runs x 11 points.\n",checks,n);
 return 0;
}
