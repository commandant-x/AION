#include "aion_sim.h"
/* Deliberately freestanding: no heap, no CRT dependencies, no platform APIs.
   x86-64 SSE2 is guaranteed for shipped Windows binary. The same source
   compiles with GCC/Clang on Linux for tests. */
static double sq(double x){return __builtin_sqrt(x);}
static double absd(double x){return x>=0?x:-x;}
static int finite_num(double x){return x==x && x<1e100 && x> -1e100;}
static V3 vec(double x,double y,double z){V3 v={x,y,z};return v;}
static V3 plus(V3 a,V3 b){return vec(a.x+b.x,a.y+b.y,a.z+b.z);}
static V3 minus(V3 a,V3 b){return vec(a.x-b.x,a.y-b.y,a.z-b.z);}
static V3 mul(V3 a,double c){return vec(a.x*c,a.y*c,a.z*c);}
static double dot(V3 a,V3 b){return a.x*b.x+a.y*b.y+a.z*b.z;}
static double length(V3 a){return sq(dot(a,a));}
static int add_node(AionSim *s,double x,double y,double z,double radius,double mass,double temperature,int kind){
  if(s->node_count>=AION_MAX_NODES)return -1;
  int i=s->node_count++;Node *n=&s->nodes[i];
  n->p=vec(x,y,z);n->v=vec(0,0,0);n->r=radius;n->m=mass;n->t=temperature;
  n->cv=mass*(kind==1?1650.:780.);n->kind=kind;n->component=i;
  return i;
}
static void add_edge(AionSim *s,int a,int b,double k,double limit){
  if(s->edge_count>=AION_MAX_BONDS || a<0||b<0 || a==b)return;
  Edge *e=&s->edges[s->edge_count++];e->a=a;e->b=b;e->alive=1;
  e->rest=length(minus(s->nodes[a].p,s->nodes[b].p));
  e->k=k;e->limit=limit;e->conductance=3.2;
  e->original_rest=e->rest;
  /* Distinct coarse-grained material responses, not atomistic constants. */
  e->yield_strain=s->material_mode==0?.10:(s->material_mode==1?.055:.20);
  e->plastic_rate=s->material_mode==0?2.5:(s->material_mode==1?8.:0.);
  e->damage_onset=s->material_mode==0?.065:(s->material_mode==1?.10:.042);
  e->damage_rate=s->material_mode==0?2.5:(s->material_mode==1?.65:8.);
  e->dashpot=s->material_mode==0?.55:(s->material_mode==1?1.5:.22);
}
/* Full initialization guarantees that a Rust MaybeUninit<AionSim> is sound. */
void aion_reset(AionSim *s,int mode){
  if(!s)return;
  volatile unsigned char *raw=(volatile unsigned char*)s;
  for(unsigned i=0;i<sizeof(*s);++i)raw[i]=0;
  s->gravity=-AION_G;s->floor_enabled=1;
  s->material_mode=(mode>=0&&mode<=2)?mode:0;
  double stiffness=s->material_mode==0?1500.:(s->material_mode==1?600.:2500.);
  double limit=s->material_mode==0?62.:(s->material_mode==1?38.:34.);
  double mass=s->material_mode==0?1.2:(s->material_mode==1?.65:1.4);
  int ids[6][4][3];
  for(int x=0;x<6;x++)for(int y=0;y<4;y++)for(int z=0;z<3;z++)
    ids[x][y][z]=add_node(s,(x-2.5)*.23,1.1+y*.23,(z-1)*.23,.070,mass,293.,s->material_mode);
  for(int x=0;x<6;x++)for(int y=0;y<4;y++)for(int z=0;z<3;z++){
    int a=ids[x][y][z];
    if(x<5)add_edge(s,a,ids[x+1][y][z],stiffness,limit);
    if(y<3)add_edge(s,a,ids[x][y+1][z],stiffness,limit);
    if(z<2)add_edge(s,a,ids[x][y][z+1],stiffness,limit);
    if(x<5&&y<3){add_edge(s,a,ids[x+1][y+1][z],stiffness*.56,limit*.7);add_edge(s,ids[x+1][y][z],ids[x][y+1][z],stiffness*.56,limit*.7);}
    if(x<5&&z<2){add_edge(s,a,ids[x+1][y][z+1],stiffness*.56,limit*.7);add_edge(s,ids[x+1][y][z],ids[x][y][z+1],stiffness*.56,limit*.7);}
    if(y<3&&z<2){add_edge(s,a,ids[x][y+1][z+1],stiffness*.56,limit*.7);add_edge(s,ids[x][y+1][z],ids[x][y][z+1],stiffness*.56,limit*.7);}
  }
  int hammer=add_node(s,.12,3.90,.01,.23,11.,293.,3);
  s->nodes[hammer].v=vec(0,-4.,0.);
}
static void add_heat(AionSim *s,int a,int b,double joules){
  if(joules<=0)return;
  double rise=joules/(s->nodes[a].cv+s->nodes[b].cv);
  s->nodes[a].t+=rise;s->nodes[b].t+=rise;
}
static void components(AionSim *s){
  int parent[AION_MAX_NODES];
  for(int i=0;i<s->node_count;i++)parent[i]=i;
  /* Plain union-find, acceptable at local laboratory scale. */
  for(int i=0;i<s->edge_count;i++)if(s->edges[i].alive){
    int a=s->edges[i].a,b=s->edges[i].b;
    while(parent[a]!=a){parent[a]=parent[parent[a]];a=parent[a];}
    while(parent[b]!=b){parent[b]=parent[parent[b]];b=parent[b];}
    if(a!=b)parent[b]=a;
  }
  for(int i=0;i<s->node_count;i++){
    int a=i;while(parent[a]!=a){parent[a]=parent[parent[a]];a=parent[a];}
    s->nodes[i].component=a;
  }
}
static void substep(AionSim *s,double dt){
  V3 force[AION_MAX_NODES];
  for(int i=0;i<s->node_count;i++)force[i]=vec(0,s->gravity*s->nodes[i].m,0);
  for(int i=0;i<s->edge_count;i++){
    Edge *e=&s->edges[i];if(!e->alive)continue;
    Node *a=&s->nodes[e->a],*b=&s->nodes[e->b];
    V3 d=minus(b->p,a->p);double dist=length(d);
    if(dist<=1e-12)continue;
    V3 n=mul(d,1./dist);
    double stretch=dist-e->rest, strain=e->rest>1e-12?absd(stretch/e->rest):0.;
    if(strain>s->max_strain)s->max_strain=strain;
    /* A plastic spring permanently changes its unloaded length once past
       yield. The reduction in elastic potential becomes local internal heat. */
    if(e->plastic_rate>0. && strain>e->yield_strain){
      double before=.5*e->k*(1.-e->damage)*stretch*stretch;
      double target=dist-(stretch>0.?1.:-1.)*e->yield_strain*e->rest;
      double alpha=e->plastic_rate*dt;if(alpha>1.)alpha=1.;
      e->rest+=(target-e->rest)*alpha;
      stretch=dist-e->rest;
      double after=.5*e->k*(1.-e->damage)*stretch*stretch;
      if(before>after){double q=before-after;add_heat(s,e->a,e->b,q);s->plastic_heat+=q;}
      s->plastic_events++;
    }
    /* Cumulative scalar fatigue/softening. Damage cannot heal by itself. */
    strain=e->rest>1e-12?absd(stretch/e->rest):0.;
    if(e->damage_rate>0. && strain>e->damage_onset){
      double old=e->damage;
      e->damage+=e->damage_rate*(strain-e->damage_onset)*dt;
      if(e->damage>1.)e->damage=1.;
      if(e->damage>old){
        double q=.5*e->k*(e->damage-old)*stretch*stretch;
        add_heat(s,e->a,e->b,q);s->damage_heat+=q;
      }
    }
    if(e->damage>s->max_damage)s->max_damage=e->damage;
    double f=e->k*(1.-e->damage)*stretch;
    if(absd(f)>e->limit || e->damage>=.999){
      if(e->damage>=.999)s->fatigue_fractures++;
      e->alive=0;s->fractures++;
      double q=.5*e->k*(1.-e->damage)*stretch*stretch;
      add_heat(s,e->a,e->b,q);s->damage_heat+=q;
      continue;
    }
    /* Axial spring + Kelvin dashpot; damping power is dissipated as heat.
       The time-discretized integrator has numerical drift (tracked in tests). */
    double rel=dot(minus(b->v,a->v),n);
    double visc=e->dashpot*rel;
    V3 spring=mul(n,f-visc);
    force[e->a]=plus(force[e->a],spring);force[e->b]=minus(force[e->b],spring);
    double dissipated=e->dashpot*rel*rel*dt;
    if(dissipated>0.){add_heat(s,e->a,e->b,dissipated);s->damping_heat+=dissipated;}
    /* Exchange is capped at equilibrium to avoid unphysical oscillations. */
    double delta=a->t-b->t,cap=a->cv*b->cv/(a->cv+b->cv);
    double q=e->conductance*delta*dt,limit=absd(delta)*cap;
    if(q>limit)q=limit;
    if(q< -limit)q=-limit;
    a->t-=q/a->cv;b->t+=q/b->cv;
  }
  for(int i=0;i<s->node_count;i++){
    Node *p=&s->nodes[i];p->v=plus(p->v,mul(force[i],dt/p->m));p->p=plus(p->p,mul(p->v,dt));
  }
  /* Only different connected bodies collide. Avoids bonded lattice points
     repelling each other while still allowing separated fragments to collide. */
  components(s);
  int sorted[AION_MAX_NODES];
  double largest_radius=0;
  for(int i=0;i<s->node_count;i++){sorted[i]=i;if(s->nodes[i].r>largest_radius)largest_radius=s->nodes[i].r;}
  for(int i=1;i<s->node_count;i++){
    int id=sorted[i],j=i;
    while(j>0 && (s->nodes[sorted[j-1]].p.x > s->nodes[id].p.x)){sorted[j]=sorted[j-1];j--;}
    sorted[j]=id;
  }
  for(int ai=0;ai<s->node_count;ai++)for(int bi=ai+1;bi<s->node_count;bi++){
    int i=sorted[ai],j=sorted[bi];Node *a=&s->nodes[i],*b=&s->nodes[j];
    if(b->p.x-a->p.x>2*largest_radius)break;
    if(a->component==b->component)continue;
    V3 d=minus(b->p,a->p);double rr=a->r+b->r;
    if(absd(d.y)>=rr || absd(d.z)>=rr || absd(d.x)>=rr)continue;
    double dist=length(d);if(dist>=rr)continue;
    V3 n=dist>1e-10?mul(d,1./dist):vec(1,0,0);
    double ia=1./a->m,ib=1./b->m;
    V3 fix=mul(n,(rr-dist)/(ia+ib));a->p=minus(a->p,mul(fix,ia));b->p=plus(b->p,mul(fix,ib));
    double rel=dot(minus(b->v,a->v),n);
    if(rel<0){
      double eb=.5*a->m*dot(a->v,a->v)+.5*b->m*dot(b->v,b->v);
      double jimp=-(1.+.24)*rel/(ia+ib);
      a->v=minus(a->v,mul(n,jimp*ia));b->v=plus(b->v,mul(n,jimp*ib));
      double ea=.5*a->m*dot(a->v,a->v)+.5*b->m*dot(b->v,b->v);
      add_heat(s,i,j,eb-ea);
      if(jimp>s->max_contact_impulse)s->max_contact_impulse=jimp;
      s->contacts++;
    }
  }
  if(s->floor_enabled)for(int i=0;i<s->node_count;i++){
    Node *p=&s->nodes[i];if(p->p.y>=p->r)continue;
    p->p.y=p->r;
    if(p->v.y<0){
      double eb=.5*p->m*dot(p->v,p->v);
      p->v.y=-p->v.y*.2;p->v.x*=.91;p->v.z*=.91;
      double ea=.5*p->m*dot(p->v,p->v);
      p->t+=(eb-ea)/p->cv;
    }
  }
  s->sim_time+=dt;
}
void aion_step(AionSim *s,double dt){
  if(!s)return;
  if(!finite_num(dt)||dt<=0||dt>.1){s->rejected_steps++;return;}
  int sub=(int)(dt/.002+0.999999);if(sub<1)sub=1;
  double h=dt/sub;
  for(int k=0;k<sub;k++)substep(s,h);
}
void aion_impact(AionSim *s){if(s && s->node_count>0)s->nodes[s->node_count-1].v.y-=7.;}
void aion_pull(AionSim *s){
  if(!s)return;
  for(int i=0;i<s->node_count;i++){
    Node *p=&s->nodes[i];
    if(p->kind==3)continue;
    if(p->p.x<-.35)p->v.x-=1.8;
    else if(p->p.x>.35)p->v.x+=1.8;
  }
}
double aion_max_plastic_strain(const AionSim *s){
  double max=0.;if(!s)return max;
  for(int i=0;i<s->edge_count;i++){
    const Edge *e=&s->edges[i];
    if(e->original_rest<1e-12)continue;
    double val=absd((e->rest-e->original_rest)/e->original_rest);
    if(val>max)max=val;
  }
  return max;
}
void aion_heat(AionSim *s){if(!s)return;for(int i=0;i<s->node_count-1;i++)if(s->nodes[i].p.x<-.25)s->nodes[i].t+=25.;}
int aion_alive_bonds(const AionSim *s){int n=0;for(int i=0;i<s->edge_count;i++)n+=s->edges[i].alive;return n;}
double aion_mass(const AionSim *s){double x=0;for(int i=0;i<s->node_count;i++)x+=s->nodes[i].m;return x;}
double aion_thermal_energy(const AionSim *s){double x=0;for(int i=0;i<s->node_count;i++)x+=s->nodes[i].t*s->nodes[i].cv;return x;}
double aion_mechanical_energy(const AionSim *s){
  double x=0;
  for(int i=0;i<s->node_count;i++){const Node *p=&s->nodes[i];x+=.5*p->m*dot(p->v,p->v)-p->m*s->gravity*p->p.y;}
  for(int i=0;i<s->edge_count;i++)if(s->edges[i].alive){const Edge *e=&s->edges[i];double ext=length(minus(s->nodes[e->a].p,s->nodes[e->b].p))-e->rest;x+=.5*e->k*ext*ext;}
  return x;
}
double aion_total_energy(const AionSim *s){return aion_mechanical_energy(s)+aion_thermal_energy(s);}
unsigned int aion_abi_size(void){return (unsigned int)sizeof(AionSim);}
static unsigned long long hashbytes(unsigned long long h,const void *p,unsigned n){
  const unsigned char *s=(const unsigned char*)p;
  for(unsigned i=0;i<n;i++){h^=(unsigned long long)s[i];h*=1099511628211ULL;}return h;
}
unsigned long long aion_checksum(const AionSim *s){
  unsigned long long h=1469598103934665603ULL;
  h=hashbytes(h,&s->node_count,sizeof(s->node_count));
  h=hashbytes(h,&s->edge_count,sizeof(s->edge_count));
  h=hashbytes(h,&s->sim_time,sizeof(s->sim_time));
  for(int i=0;i<s->node_count;i++){
    const Node *p=&s->nodes[i];h=hashbytes(h,&p->p,sizeof(p->p));h=hashbytes(h,&p->v,sizeof(p->v));h=hashbytes(h,&p->t,sizeof(p->t));
  }
  for(int i=0;i<s->edge_count;i++){
    const Edge *e=&s->edges[i];
    h=hashbytes(h,&e->alive,sizeof(e->alive));
    h=hashbytes(h,&e->rest,sizeof(e->rest));
    h=hashbytes(h,&e->damage,sizeof(e->damage));
  }
  return h;
}

/* Laboratory checkpoint 0.6: raw ABI state wrapped with a fixed-size header.
   It is not guaranteed portable across architectures or compiler struct packing.
   The header uses explicit little-endian scalars and a 64-bit integrity hash. */
#define AION_CK_HEADER 32u
static void put32(unsigned char *p,unsigned int v){for(int i=0;i<4;i++)p[i]=(unsigned char)(v>>(8*i));}
static void put64(unsigned char *p,unsigned long long v){for(int i=0;i<8;i++)p[i]=(unsigned char)(v>>(8*i));}
static unsigned int get32(const unsigned char *p){unsigned int v=0;for(int i=0;i<4;i++)v|=((unsigned int)p[i])<<(8*i);return v;}
static unsigned long long get64(const unsigned char *p){unsigned long long v=0;for(int i=0;i<8;i++)v|=((unsigned long long)p[i])<<(8*i);return v;}
static unsigned long long checksum_data(const unsigned char*p,unsigned n){
 unsigned long long h=1469598103934665603ULL;
 for(unsigned i=0;i<n;i++){h^=p[i];h*=1099511628211ULL;}return h;
}
unsigned int aion_checkpoint_bytes(void){return AION_CK_HEADER+(unsigned int)sizeof(AionSim);}
unsigned int aion_checkpoint_write(const AionSim*s,unsigned char*out,unsigned int cap){
 if(!s||!out||cap<aion_checkpoint_bytes())return 0;
 static const unsigned char magic[8]={'A','I','O','N','C','K','0','6'};
 for(int i=0;i<8;i++)out[i]=magic[i];
 put32(out+8,6);put32(out+12,(unsigned int)sizeof(AionSim));
 const unsigned char*src=(const unsigned char*)s;
 for(unsigned i=0;i<sizeof(AionSim);i++)out[AION_CK_HEADER+i]=src[i];
 put64(out+16,checksum_data(out+AION_CK_HEADER,(unsigned)sizeof(AionSim)));
 put32(out+24,0x01020304u);put32(out+28,0);
 return aion_checkpoint_bytes();
}
int aion_checkpoint_read(AionSim*dst,const unsigned char*bytes,unsigned int length){
 static const unsigned char magic[8]={'A','I','O','N','C','K','0','6'};
 if(!dst||!bytes||length!=aion_checkpoint_bytes())return 0;
 for(int i=0;i<8;i++)if(bytes[i]!=magic[i])return 0;
 if(get32(bytes+8)!=6||get32(bytes+12)!=(unsigned int)sizeof(AionSim)||get32(bytes+24)!=0x01020304u||get32(bytes+28)!=0)return 0;
 if(get64(bytes+16)!=checksum_data(bytes+AION_CK_HEADER,(unsigned)sizeof(AionSim)))return 0;
 static AionSim candidate;
 volatile unsigned char *p=(volatile unsigned char*)&candidate;
 for(unsigned i=0;i<sizeof(AionSim);i++)p[i]=bytes[AION_CK_HEADER+i];
 if(candidate.node_count<0||candidate.node_count>AION_MAX_NODES||candidate.edge_count<0||candidate.edge_count>AION_MAX_BONDS||
  !finite_num(candidate.sim_time)||candidate.sim_time<0||!finite_num(candidate.gravity)||candidate.gravity< -100000.||candidate.gravity>100000.)return 0;
 for(int i=0;i<candidate.node_count;i++){
   const Node*n=&candidate.nodes[i];
   if(!finite_num(n->p.x)||!finite_num(n->p.y)||!finite_num(n->p.z)||
      !finite_num(n->v.x)||!finite_num(n->v.y)||!finite_num(n->v.z)||
      !finite_num(n->t)||n->t<0||!finite_num(n->r)||n->r<=0||!finite_num(n->m)||n->m<=0||
      !finite_num(n->cv)||n->cv<=0||n->component<0||n->component>=candidate.node_count)return 0;
 }
 for(int i=0;i<candidate.edge_count;i++){
   const Edge*e=&candidate.edges[i];
   if(e->a<0||e->a>=candidate.node_count||e->b<0||e->b>=candidate.node_count||e->alive<0||e->alive>1||
      !finite_num(e->rest)||e->rest<=0||!finite_num(e->k)||e->k<0||!finite_num(e->limit)||e->limit<0||
      !finite_num(e->damage)||e->damage<0||e->damage>1||!finite_num(e->conductance)||e->conductance<0)return 0;
 }
 volatile unsigned char *out=(volatile unsigned char*)dst;
 for(unsigned i=0;i<sizeof(AionSim);i++)out[i]=p[i];
 return 1;
}
