/* AION CORE 0.7: canonical, ABI-independent mechanics checkpoints.
   Windows freestanding compatible: no CRT heap, memcpy, FILE*, malloc or math library. */
#include "checkpoint_portable.h"
#include <stdint.h>

typedef char require_32bit_int[(sizeof(int)==4)?1:-1];
typedef char require_binary64[(sizeof(double)==8)?1:-1];
static unsigned char const MAGIC[8]={'A','I','O','N','S','0','0','7'};
static void w32(unsigned char*p,uint32_t n){for(unsigned i=0;i<4;i++)p[i]=(unsigned char)(n>>(8*i));}
static void w64(unsigned char*p,uint64_t n){for(unsigned i=0;i<8;i++)p[i]=(unsigned char)(n>>(8*i));}
static uint32_t r32(const unsigned char*p){uint32_t v=0;for(unsigned i=0;i<4;i++)v|=(uint32_t)p[i]<<(8*i);return v;}
static uint64_t r64(const unsigned char*p){uint64_t v=0;for(unsigned i=0;i<8;i++)v|=(uint64_t)p[i]<<(8*i);return v;}
static void wf(unsigned char*p,double f){union{double f;uint64_t u;}x;x.f=f;w64(p,x.u);}
static double rf(const unsigned char*p){union{double f;uint64_t u;}x;x.u=r64(p);return x.f;}
static uint64_t checksum(const unsigned char*p,unsigned n){uint64_t h=1469598103934665603ULL;for(unsigned i=0;i<n;i++){h^=p[i];h*=1099511628211ULL;}return h;}
static int finite(double x){return x==x && x<1.e300 && x> -1.e300;}
static void put_i(unsigned char *b,unsigned *pos,int v){w32(b+*pos,(uint32_t)v);*pos+=4;}
static void put_f(unsigned char *b,unsigned *pos,double v){wf(b+*pos,v);*pos+=8;}
static int get_i(const unsigned char *b,unsigned *pos){int v=(int)(int32_t)r32(b+*pos);*pos+=4;return v;}
static double get_f(const unsigned char*b,unsigned*pos){double v=rf(b+*pos);*pos+=8;return v;}
static int valid(const AionSim *s){
 if(s->node_count<0||s->node_count>AION_MAX_NODES||s->edge_count<0||s->edge_count>AION_MAX_BONDS)return 0;
 if(s->fractures<0||s->contacts<0||s->rejected_steps<0||s->plastic_events<0||s->fatigue_fractures<0||s->floor_enabled<0||s->floor_enabled>1||s->material_mode<0||s->material_mode>2)return 0;
 const double meta[]={s->sim_time,s->gravity,s->max_strain,s->max_contact_impulse,s->plastic_heat,s->damage_heat,s->damping_heat,s->max_damage};
 for(unsigned i=0;i<8;i++)if(!finite(meta[i]))return 0;
 if(s->sim_time<0||s->max_strain<0||s->max_contact_impulse<0||s->plastic_heat<0||s->damage_heat<0||s->damping_heat<0||s->max_damage<0||s->max_damage>1)return 0;
 for(int i=0;i<s->node_count;i++){
  const Node*n=s->nodes+i;
  if(!finite(n->p.x)||!finite(n->p.y)||!finite(n->p.z)||!finite(n->v.x)||!finite(n->v.y)||!finite(n->v.z)||!finite(n->r)||!finite(n->m)||!finite(n->t)||!finite(n->cv))return 0;
  if(n->r<=0||n->m<=0||n->t<0||n->cv<=0||n->component<0||n->component>=s->node_count||n->kind<0||n->kind>3)return 0;
 }
 for(int i=0;i<s->edge_count;i++){
  const Edge*e=s->edges+i;
  if(e->a<0||e->a>=s->node_count||e->b<0||e->b>=s->node_count||e->a==e->b||e->alive<0||e->alive>1)return 0;
  const double v[]={e->rest,e->k,e->limit,e->conductance,e->original_rest,e->yield_strain,e->plastic_rate,e->damage,e->damage_onset,e->damage_rate,e->dashpot};
  for(unsigned j=0;j<11;j++)if(!finite(v[j]))return 0;
  if(e->rest<=0||e->original_rest<=0||e->k<0||e->limit<0||e->conductance<0||e->yield_strain<0||e->plastic_rate<0||e->damage<0||e->damage>1||e->damage_onset<0||e->damage_rate<0||e->dashpot<0)return 0;
 }
 return 1;
}
unsigned aion7_bytes(const AionSim *s){
 if(!s||s->node_count<0||s->node_count>AION_MAX_NODES||s->edge_count<0||s->edge_count>AION_MAX_BONDS)return 0;
 return AION7_HEADER_BYTES+AION7_FIXED_BYTES+(unsigned)s->node_count*AION7_NODE_BYTES+(unsigned)s->edge_count*AION7_EDGE_BYTES;
}
unsigned aion7_write(const AionSim*s,unsigned char*out,unsigned cap){
 unsigned len=aion7_bytes(s);if(!len||!out||cap<len||!valid(s))return 0;
 for(unsigned i=0;i<8;i++){out[i]=MAGIC[i];}
 w32(out+8,7);w32(out+12,len);w64(out+16,0);w32(out+24,0x01020304u);w32(out+28,0);
 unsigned p=AION7_HEADER_BYTES;
 put_i(out,&p,s->node_count);put_i(out,&p,s->edge_count);put_i(out,&p,s->fractures);put_i(out,&p,s->contacts);put_i(out,&p,s->material_mode);put_i(out,&p,s->rejected_steps);put_i(out,&p,s->plastic_events);put_i(out,&p,s->fatigue_fractures);put_i(out,&p,s->floor_enabled);
 put_f(out,&p,s->sim_time);put_f(out,&p,s->gravity);put_f(out,&p,s->max_strain);put_f(out,&p,s->max_contact_impulse);put_f(out,&p,s->plastic_heat);put_f(out,&p,s->damage_heat);put_f(out,&p,s->damping_heat);put_f(out,&p,s->max_damage);
 for(int i=0;i<s->node_count;i++){
  const Node*n=s->nodes+i;
  put_f(out,&p,n->p.x);put_f(out,&p,n->p.y);put_f(out,&p,n->p.z);
  put_f(out,&p,n->v.x);put_f(out,&p,n->v.y);put_f(out,&p,n->v.z);
  put_f(out,&p,n->r);put_f(out,&p,n->m);put_f(out,&p,n->t);put_f(out,&p,n->cv);
  put_i(out,&p,n->kind);put_i(out,&p,n->component);
 }
 for(int i=0;i<s->edge_count;i++){
  const Edge*e=s->edges+i;
  put_i(out,&p,e->a);put_i(out,&p,e->b);put_i(out,&p,e->alive);
  put_f(out,&p,e->rest);put_f(out,&p,e->k);put_f(out,&p,e->limit);put_f(out,&p,e->conductance);
  put_f(out,&p,e->original_rest);put_f(out,&p,e->yield_strain);put_f(out,&p,e->plastic_rate);put_f(out,&p,e->damage);put_f(out,&p,e->damage_onset);put_f(out,&p,e->damage_rate);put_f(out,&p,e->dashpot);
 }
 if(p!=len)return 0;
 w64(out+16,checksum(out+AION7_HEADER_BYTES,len-AION7_HEADER_BYTES));return len;
}
int aion7_read(AionSim*dst,const unsigned char*input,unsigned len){
 if(!dst||!input||len<AION7_HEADER_BYTES+AION7_FIXED_BYTES||len>AION7_MAX_BYTES)return 0;
 for(unsigned i=0;i<8;i++)if(input[i]!=MAGIC[i])return 0;
 if(r32(input+8)!=7||r32(input+12)!=len||r32(input+24)!=0x01020304u||r32(input+28)!=0)return 0;
 if(r64(input+16)!=checksum(input+AION7_HEADER_BYTES,len-AION7_HEADER_BYTES))return 0;
 unsigned n=r32(input+32),e=r32(input+36);
 if(n>AION_MAX_NODES||e>AION_MAX_BONDS||len!=AION7_HEADER_BYTES+AION7_FIXED_BYTES+n*AION7_NODE_BYTES+e*AION7_EDGE_BYTES)return 0;
 /* Temporary scratch is static for freestanding Win32 (avoids __chkstk).
    Reader must not be called concurrently; future Rust decoder is re-entrant. */
 static AionSim tmp;unsigned p=AION7_HEADER_BYTES;
 /* Reset all inactive fields to deterministic zero without relying on libc. */
 volatile unsigned char *raw=(volatile unsigned char*)&tmp;
 for(unsigned i=0;i<sizeof(tmp);i++)raw[i]=0;
 tmp.node_count=get_i(input,&p);tmp.edge_count=get_i(input,&p);tmp.fractures=get_i(input,&p);tmp.contacts=get_i(input,&p);tmp.material_mode=get_i(input,&p);tmp.rejected_steps=get_i(input,&p);tmp.plastic_events=get_i(input,&p);tmp.fatigue_fractures=get_i(input,&p);tmp.floor_enabled=get_i(input,&p);
 tmp.sim_time=get_f(input,&p);tmp.gravity=get_f(input,&p);tmp.max_strain=get_f(input,&p);tmp.max_contact_impulse=get_f(input,&p);tmp.plastic_heat=get_f(input,&p);tmp.damage_heat=get_f(input,&p);tmp.damping_heat=get_f(input,&p);tmp.max_damage=get_f(input,&p);
 for(unsigned i=0;i<n;i++){
  Node*x=tmp.nodes+i;
  x->p.x=get_f(input,&p);x->p.y=get_f(input,&p);x->p.z=get_f(input,&p);
  x->v.x=get_f(input,&p);x->v.y=get_f(input,&p);x->v.z=get_f(input,&p);
  x->r=get_f(input,&p);x->m=get_f(input,&p);x->t=get_f(input,&p);x->cv=get_f(input,&p);
  x->kind=get_i(input,&p);x->component=get_i(input,&p);
 }
 for(unsigned i=0;i<e;i++){
  Edge*x=tmp.edges+i;
  x->a=get_i(input,&p);x->b=get_i(input,&p);x->alive=get_i(input,&p);
  x->rest=get_f(input,&p);x->k=get_f(input,&p);x->limit=get_f(input,&p);x->conductance=get_f(input,&p);
  x->original_rest=get_f(input,&p);x->yield_strain=get_f(input,&p);x->plastic_rate=get_f(input,&p);x->damage=get_f(input,&p);x->damage_onset=get_f(input,&p);x->damage_rate=get_f(input,&p);x->dashpot=get_f(input,&p);
 }
 if(p!=len||!valid(&tmp))return 0;
 volatile unsigned char *d=(volatile unsigned char*)dst;
 for(unsigned i=0;i<sizeof(tmp);i++)d[i]=raw[i];
 return 1;
}
