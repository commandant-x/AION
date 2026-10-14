#include "../native/aion_sim.h"
#include "../native/checkpoint_portable.h"
#include <stdio.h>
static AionSim world;
static unsigned char data[AION7_MAX_BYTES];
int main(int argc,char**argv){
 if(argc!=2)return 2;
 aion_reset(&world,1);
 for(int i=0;i<350;i++)aion_step(&world,.003);
 unsigned n=aion7_write(&world,data,sizeof(data));if(!n)return 3;
 FILE*f=fopen(argv[1],"wb");if(!f)return 4;
 if(fwrite(data,1,n,f)!=n){fclose(f);return 5;}
 fclose(f);printf("Wrote %u bytes\n",n);return 0;
}
