#include "../native/aion_sim.h"
#include "../native/checkpoint_portable.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define TEST(x) do{if(!(x)){fprintf(stderr,"FAILED line %d: %s\n",__LINE__,#x);return 1;}checks++;}while(0)
static unsigned char data[AION7_MAX_BYTES],again[AION7_MAX_BYTES];
static AionSim a,b,safe;
static int checks;
static void rehash(unsigned char *buf,unsigned n){
 unsigned long long h=1469598103934665603ULL;
 for(unsigned i=32;i<n;i++){h^=buf[i];h*=1099511628211ULL;}
 for(unsigned i=0;i<8;i++)buf[16+i]=(unsigned char)(h>>(8*i));
}
int main(void){
 TEST(AION7_MAX_BYTES<200000u);
 for(int mode=0;mode<3;mode++){
  aion_reset(&a,mode);
  TEST(a.node_count==73&&a.edge_count==404);
  for(int i=0;i<1750;i++)aion_step(&a,.003);
  unsigned n=aion7_write(&a,data,sizeof(data));
  TEST(n==aion7_bytes(&a));
  TEST(n<sizeof(AionSim));
  TEST(data[0]=='A'&&data[7]=='7');
  TEST(aion7_read(&b,data,n));
  TEST(aion_checksum(&b)==aion_checksum(&a));
  TEST(aion7_write(&b,again,sizeof(again))==n);
  TEST(memcmp(data,again,n)==0); /* canonical byte-for-byte roundtrip */
  for(int i=0;i<900;i++){aion_step(&a,.003);aion_step(&b,.003);}
  TEST(aion_checksum(&a)==aion_checksum(&b));
  safe=b;unsigned long long previous=aion_checksum(&safe);
  TEST(!aion7_read(&safe,data,n-1));TEST(aion_checksum(&safe)==previous);
  data[90]^=0x20;TEST(!aion7_read(&safe,data,n));data[90]^=0x20;
  data[8]^=1;TEST(!aion7_read(&safe,data,n));data[8]^=1;
  data[32]=255;rehash(data,n);TEST(!aion7_read(&safe,data,n));
  TEST(aion_checksum(&safe)==previous);
  TEST(aion7_write(&a,data,1)==0);
  TEST(aion7_write(0,data,n)==0);
  TEST(aion7_read(0,data,n)==0);
  TEST(aion7_read(&safe,0,n)==0);
  /* Repair buffer by regenerating valid checkpoint. */
  TEST(aion7_write(&b,data,sizeof(data))>0);
 }
 /* A record with valid hash but physically impossible mass must fail. */
 aion_reset(&a,0);unsigned n=aion7_write(&a,data,sizeof(data));
 TEST(n>200);
 /* First node mass offset: header(32)+fixed(100)+7 doubles(56), i.e. 188 */
 for(int i=0;i<8;i++)data[32+100+7*8+i]=0;
 rehash(data,n);
 aion_reset(&safe,2);unsigned long long prev=aion_checksum(&safe);
 TEST(!aion7_read(&safe,data,n));TEST(aion_checksum(&safe)==prev);
 /* Branching experiments must diverge after distinct stimuli. */
 aion_reset(&a,0);for(int i=0;i<100;i++)aion_step(&a,.003);
 n=aion7_write(&a,data,sizeof(data));TEST(aion7_read(&b,data,n));
 aion_pull(&a);aion_heat(&b);
 for(int i=0;i<500;i++){aion_step(&a,.003);aion_step(&b,.003);}
 TEST(aion_checksum(&a)!=aion_checksum(&b));
 printf("AION7 portable checkpoint: %d assertions PASS.\n",checks);
 return 0;
}
