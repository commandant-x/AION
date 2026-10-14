#include "../native/aion_sim.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#define VERIFY(x) do{if(!(x)){fprintf(stderr,"FAIL %s:%d %s\n",__FILE__,__LINE__,#x);return 1;}passed++;}while(0)
static AionSim a,b,unchanged,fake;
static void rehash(unsigned char *buf){
  unsigned long long h=1469598103934665603ULL;
  for(unsigned i=32;i<sizeof(AionSim)+32;i++){h^=buf[i];h*=1099511628211ULL;}
  for(int i=0;i<8;i++)buf[16+i]=(unsigned char)(h>>(8*i));
}
static unsigned char memory[sizeof(AionSim)+32];
static int passed;
int main(void){
  VERIFY(aion_checkpoint_bytes()==sizeof(memory));
  aion_reset(&a,0);for(int i=0;i<1800;i++)aion_step(&a,.003);
  unsigned long long before=aion_checksum(&a);
  VERIFY(aion_checkpoint_write(&a,memory,sizeof(memory))==sizeof(memory));
  VERIFY(aion_checkpoint_read(&b,memory,sizeof(memory))==1);
  VERIFY(aion_checksum(&b)==before);
  for(int i=0;i<900;i++){aion_step(&a,.003);aion_step(&b,.003);}
  VERIFY(aion_checksum(&a)==aion_checksum(&b));
  aion_reset(&unchanged,2);unsigned long long safe=aion_checksum(&unchanged);
  VERIFY(aion_checkpoint_read(&unchanged,memory,sizeof(memory)-1)==0);
  VERIFY(aion_checksum(&unchanged)==safe);
  memory[8]^=1;VERIFY(aion_checkpoint_read(&unchanged,memory,sizeof(memory))==0);memory[8]^=1;
  memory[44]^=1;VERIFY(aion_checkpoint_read(&unchanged,memory,sizeof(memory))==0);memory[44]^=1;
  VERIFY(aion_checkpoint_read(&unchanged,memory,sizeof(memory))==1);
  VERIFY(aion_checksum(&unchanged)==before);
  VERIFY(aion_checkpoint_write(&a,memory,sizeof(memory)-1)==0);
  /* Valid checksums alone do not authorize physically invalid states. */
  fake=unchanged;fake.nodes[0].m=-1.;
  memcpy(memory+32,&fake,sizeof(fake));rehash(memory);
  VERIFY(aion_checkpoint_read(&b,memory,sizeof(memory))==0);
  fake=unchanged;fake.edges[0].a=4000;
  memcpy(memory+32,&fake,sizeof(fake));rehash(memory);
  VERIFY(aion_checkpoint_read(&b,memory,sizeof(memory))==0);
  /* Branch two identical experiments after restoring the same checkpoint. */
  aion_reset(&a,0);for(int i=0;i<200;i++)aion_step(&a,.003);
  VERIFY(aion_checkpoint_write(&a,memory,sizeof(memory))==sizeof(memory));
  VERIFY(aion_checkpoint_read(&b,memory,sizeof(memory))==1);
  aion_pull(&a);aion_heat(&b);
  for(int i=0;i<100;i++){aion_step(&a,.003);aion_step(&b,.003);}
  VERIFY(aion_checksum(&a)!=aion_checksum(&b));
  for(int material=0;material<3;material++){
    aion_reset(&a,material);aion_pull(&a);aion_heat(&a);
    for(int i=0;i<500;i++)aion_step(&a,.003);
    VERIFY(aion_checkpoint_write(&a,memory,sizeof(memory))==sizeof(memory));
    VERIFY(aion_checkpoint_read(&b,memory,sizeof(memory))==1);
    VERIFY(aion_checksum(&a)==aion_checksum(&b));
    VERIFY(b.material_mode==material);
  }
  printf("AION checkpoint 0.6: %d independent assertions passed.\n",passed);
  return 0;
}
