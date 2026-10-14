/* AION CORE 0.9 -- reproducible material-experiments reference preview (Rust migration in progress) -- self-contained native Windows research viewer.
   Compiled with clang/lld to x86-64 PE, using user32/gdi32/kernel32 ONLY.
   The viewer uses the shared native C kernel, also callable through Rust FFI.
*/
#include "../native/experiment.h"
#include "physics.h"
#include "../native/checkpoint_portable.h"
/* Freestanding Windows build: sqrt uses hardware SSE2, no runtime. */
int _fltused = 0;
double sqrt(double x){double y;__asm__ volatile("sqrtsd %1, %0":"=x"(y):"x"(x));return y;}

typedef void *HND;
typedef HND HANDLE;
typedef HND HWND, HDC, HBITMAP, HBRUSH, HPEN, HFONT, HGDIOBJ, HINSTANCE;
typedef unsigned int UINT, DWORD;
typedef unsigned long long ULONG_PTR, WPARAM;
typedef long long LPARAM, LRESULT;
typedef struct {int left,top,right,bottom;} RECT;
typedef struct {int x,y;} POINT;
typedef struct {HDC hdc;int fErase;RECT rcPaint;int fRestore;int fIncUpdate;unsigned char reserved[32];} PAINTSTRUCT;
typedef LRESULT (__stdcall *PROC)(HWND,UINT,WPARAM,LPARAM);
typedef struct {UINT cbSize,style;PROC lpfnWndProc;int cbClsExtra,cbWndExtra;HINSTANCE hInstance;HND hIcon,hCursor;HBRUSH hbrBackground;const char *lpszMenuName,*lpszClassName;HND hIconSm;} WNDCLASSEXA;
typedef struct {HWND hwnd;UINT message;WPARAM wParam;LPARAM lParam;DWORD time;POINT pt;DWORD lPrivate;} MSG;

__declspec(dllimport) HINSTANCE __stdcall GetModuleHandleA(const char*);
__declspec(dllimport) void __stdcall ExitProcess(UINT);
__declspec(dllimport) HANDLE __stdcall CreateFileA(const char*,DWORD,DWORD,void*,DWORD,DWORD,HANDLE);
__declspec(dllimport) int __stdcall ReadFile(HANDLE,void*,DWORD,DWORD*,void*);
__declspec(dllimport) int __stdcall WriteFile(HANDLE,const void*,DWORD,DWORD*,void*);
__declspec(dllimport) int __stdcall CloseHandle(HANDLE);
__declspec(dllimport) unsigned long long __stdcall GetTickCount64(void);
__declspec(dllimport) UINT __stdcall RegisterClassExA(const WNDCLASSEXA*);
__declspec(dllimport) HWND __stdcall CreateWindowExA(DWORD,const char*,const char*,DWORD,int,int,int,int,HWND,HND,HINSTANCE,void*);
__declspec(dllimport) LRESULT __stdcall DefWindowProcA(HWND,UINT,WPARAM,LPARAM);
__declspec(dllimport) int __stdcall ShowWindow(HWND,int);
__declspec(dllimport) HND __stdcall LoadCursorA(HINSTANCE,const char*);
__declspec(dllimport) int __stdcall UpdateWindow(HWND);
__declspec(dllimport) int __stdcall GetMessageA(MSG*,HWND,UINT,UINT);
__declspec(dllimport) int __stdcall TranslateMessage(const MSG*);
__declspec(dllimport) LRESULT __stdcall DispatchMessageA(const MSG*);
__declspec(dllimport) void __stdcall PostQuitMessage(int);
__declspec(dllimport) HDC __stdcall BeginPaint(HWND,PAINTSTRUCT*);
__declspec(dllimport) int __stdcall EndPaint(HWND,const PAINTSTRUCT*);
__declspec(dllimport) int __stdcall GetClientRect(HWND,RECT*);
__declspec(dllimport) int __stdcall InvalidateRect(HWND,const RECT*,int);
__declspec(dllimport) ULONG_PTR __stdcall SetTimer(HWND,ULONG_PTR,UINT,void*);
__declspec(dllimport) int __stdcall KillTimer(HWND,ULONG_PTR);
__declspec(dllimport) int __stdcall FillRect(HDC,const RECT*,HBRUSH);
__declspec(dllimport) HWND __stdcall SetCapture(HWND);
__declspec(dllimport) int __stdcall ReleaseCapture(void);
__declspec(dllimport) HDC __stdcall CreateCompatibleDC(HDC);
__declspec(dllimport) HBITMAP __stdcall CreateCompatibleBitmap(HDC,int,int);
__declspec(dllimport) HGDIOBJ __stdcall SelectObject(HDC,HGDIOBJ);
__declspec(dllimport) int __stdcall DeleteDC(HDC);
__declspec(dllimport) int __stdcall DeleteObject(HGDIOBJ);
__declspec(dllimport) HBRUSH __stdcall CreateSolidBrush(DWORD);
__declspec(dllimport) HPEN __stdcall CreatePen(int,int,DWORD);
__declspec(dllimport) int __stdcall MoveToEx(HDC,int,int,POINT*);
__declspec(dllimport) int __stdcall LineTo(HDC,int,int);
__declspec(dllimport) int __stdcall Ellipse(HDC,int,int,int,int);
__declspec(dllimport) int __stdcall RoundRect(HDC,int,int,int,int,int,int);
__declspec(dllimport) int __stdcall BitBlt(HDC,int,int,int,int,HDC,int,int,DWORD);
__declspec(dllimport) DWORD __stdcall SetTextColor(HDC,DWORD);
__declspec(dllimport) int __stdcall SetBkMode(HDC,int);
__declspec(dllimport) int __stdcall TextOutA(HDC,int,int,const char*,int);
__declspec(dllimport) HFONT __stdcall CreateFontA(int,int,int,int,int,DWORD,DWORD,DWORD,DWORD,DWORD,DWORD,DWORD,DWORD,const char*);

#define WS_OVERLAPPEDWINDOW 0x00CF0000u
#define WS_VISIBLE 0x10000000u
#define WM_DESTROY 0x0002
#define WM_PAINT 0x000F
#define WM_TIMER 0x0113
#define WM_KEYDOWN 0x0100
#define WM_LBUTTONDOWN 0x0201
#define WM_LBUTTONUP 0x0202
#define WM_MOUSEMOVE 0x0200
#define WM_MOUSEWHEEL 0x020A
#define WM_SIZE 0x0005
#define SW_SHOW 5
#define SRCCOPY 0x00CC0020
#define TRANSPARENT 1
#define CLR(r,g,b) ((DWORD)(((b)<<16)|((g)<<8)|(r)))
#define UIBG CLR(9,14,25)
#define PANEL CLR(15,24,39)
#define PANEL2 CLR(21,33,50)
#define LINE_CLR CLR(42,62,79)
#define TEXT CLR(228,241,250)
#define MUTED CLR(125,155,174)
#define ACCENT CLR(76,219,202)
#define GOLD CLR(255,184,100)

static HWND window;
static HFONT f_small,f_label,f_title,f_huge;
static int view_w=1300,view_h=800,mouse_drag=0,last_x=0,last_y=0;
static double yaw_s=.36,yaw_c=.933,pitch_s=.27,pitch_c=.963,zoom=1.;
static int tick_counter=0;
static int trace_count=0;
static unsigned char checkpoint_data[AION7_MAX_BYTES];
static const char*checkpoint_status="S: SAVE   L: LOAD   B: BENCHMARK";
static AionBenchResult benchmark_runs[AION_BENCH_RUNS];
static char benchmark_csv[AION_BENCH_CSV_CAP];
static void export_benchmark(void){
 /* Runs on independent scratch state; never alters the user experiment. */
 unsigned r=0;
 for(int material=0;material<3;material++)for(int protocol=0;protocol<3;protocol++){
  if(!aion_bench_run(material,protocol,&benchmark_runs[r++])){
   checkpoint_status="BENCHMARK: SIMULATION ERROR";return;
  }
 }
 unsigned len=aion_bench_csv(benchmark_runs,AION_BENCH_RUNS,benchmark_csv,sizeof(benchmark_csv));
 if(len==0){checkpoint_status="BENCHMARK: CSV SERIALIZATION FAILED";return;}
 HANDLE f=CreateFileA("AION_CORE_0_9_experiments.csv",0x40000000u,0,0,2,0,0);
 if(f==(HANDLE)(-1)||!f){checkpoint_status="BENCHMARK: FILE ERROR";return;}
 DWORD written=0;int ok=WriteFile(f,benchmark_csv,len,&written,0);
 CloseHandle(f);
 checkpoint_status=(ok && written==len)?"BENCHMARK: 9 RUNS -> CSV SAVED":"BENCHMARK: INCOMPLETE FILE";
}
static void save_checkpoint(void){
 unsigned length=aion7_write(&sim,checkpoint_data,sizeof(checkpoint_data));
 if(!length){checkpoint_status="ERROR: ENCODE FAILED";return;}
 HANDLE f=CreateFileA("AION_CORE_0_8_checkpoint.aion7",0x40000000u,0,0,2,0,0);
 if(f==(HANDLE)(-1)||!f){checkpoint_status="ERROR: CANNOT WRITE FILE";return;}
 DWORD n=0;int ok=WriteFile(f,checkpoint_data,length,&n,0);
 CloseHandle(f);checkpoint_status=(ok&&n==length)?"CHECKPOINT SAVED":"ERROR: INCOMPLETE WRITE";
}
static void load_checkpoint(void){
 HANDLE f=CreateFileA("AION_CORE_0_8_checkpoint.aion7",0x80000000u,1,0,3,0,0);
 if(f==(HANDLE)(-1)||!f){checkpoint_status="ERROR: NO CHECKPOINT";return;}
 DWORD n=0;int ok=ReadFile(f,checkpoint_data,sizeof(checkpoint_data),&n,0);
 CloseHandle(f);
 if(!ok||n<AION7_HEADER_BYTES+AION7_FIXED_BYTES){checkpoint_status="ERROR: INVALID FILE LENGTH";return;}
 if(!aion7_read(&sim,checkpoint_data,n)){checkpoint_status="ERROR: BAD CHECKPOINT";return;}
 material_mode=sim.material_mode;trace_count=0;checkpoint_status="CHECKPOINT RESTORED";
}
#define CHART_POINTS 90
static double trace_mech[CHART_POINTS];
static int trace_fract[CHART_POINTS];
static void record_trace(void){
 if(trace_count==CHART_POINTS){for(int i=1;i<CHART_POINTS;i++){
    trace_mech[i-1]=trace_mech[i];trace_fract[i-1]=trace_fract[i];
 }trace_count--;}
 trace_mech[trace_count]=aion_mechanical_energy(&sim);
 trace_fract[trace_count]=fractures;
 trace_count++;
}


static int slen(const char *s){int n=0;while(s[n])n++;return n;}
static void txt(HDC dc,int x,int y,const char*s,DWORD c,HFONT font){
 SetTextColor(dc,c);SelectObject(dc,font);TextOutA(dc,x,y,s,slen(s));
}
static void rect(HDC dc,int x,int y,int w,int h,DWORD color){
 if(w<1||h<1)return;RECT r={x,y,x+w,y+h};HBRUSH b=CreateSolidBrush(color);FillRect(dc,&r,b);DeleteObject(b);
}
static void pill(HDC dc,int x,int y,int w,int h,DWORD color){
 HBRUSH b=CreateSolidBrush(color);HGDIOBJ oldb=SelectObject(dc,b);
 HPEN p=CreatePen(0,1,color);HGDIOBJ oldp=SelectObject(dc,p);
 RoundRect(dc,x,y,x+w,y+h,12,12);
 SelectObject(dc,oldp);SelectObject(dc,oldb);DeleteObject(p);DeleteObject(b);
}
static void stroke(HDC dc,int x,int y,int xx,int yy,DWORD color,int width){
 HPEN pen=CreatePen(0,width,color);HGDIOBJ old=SelectObject(dc,pen);
 MoveToEx(dc,x,y,0);LineTo(dc,xx,yy);SelectObject(dc,old);DeleteObject(pen);
}
static void sphere(HDC dc,int x,int y,int r,DWORD color){
 if(r<=0)return;HBRUSH b=CreateSolidBrush(color);HGDIOBJ ob=SelectObject(dc,b);
 HPEN p=CreatePen(0,1,color);HGDIOBJ op=SelectObject(dc,p);
 Ellipse(dc,x-r,y-r,x+r,y+r);
 SelectObject(dc,op);SelectObject(dc,ob);DeleteObject(p);DeleteObject(b);
}
static char sbuf[200];static int bn;
static void clearstr(void){bn=0;sbuf[0]=0;}
static void append(const char*s){while(*s && bn<192)sbuf[bn++]=*s++;sbuf[bn]=0;}
static void integer(int n){char t[24];int i=0,negative=n<0;if(negative)n=-n;do{t[i++]=(char)('0'+n%10);n/=10;}while(n&&i<22);if(negative)t[i++]='-';while(i&&bn<192)sbuf[bn++]=t[--i];sbuf[bn]=0;}
static void fixed1(double x){if(x<0){append("-");x=-x;}int whole=(int)x;integer(whole);append(".");integer((int)((x-whole)*10.+.5)%10);}
static void stat(HDC dc,int x,int y,const char*label,const char*value,DWORD col){
 txt(dc,x,y,label,MUTED,f_small);txt(dc,x,y+19,value,col,f_label);
}
static int roundi(double x){return (int)(x>=0.?x+.5:x-.5);}
static void project(V3 p,int*ox,int*oy,double *depth){
 double gx=p.x*yaw_c+p.z*yaw_s;
 double gz=p.z*yaw_c-p.x*yaw_s;
 double gy=(p.y-1.5)*pitch_c-gz*pitch_s;
 double d=(p.y-1.5)*pitch_s+gz*pitch_c;
 double lens=4.8+d;if(lens<1.6)lens=1.6;
 double s=zoom*560./lens;
 *ox=240+(view_w-490)/2+roundi(gx*s);
 *oy=85+(view_h-180)/2-roundi(gy*s);
 *depth=d;
}
static void rot_pair(double*sa,double*ca,double radians){
 /* small-angle recurrence: no C runtime math functions required */
 while(radians>.08){rot_pair(sa,ca,.08);radians-=.08;}
 while(radians<-.08){rot_pair(sa,ca,-.08);radians+=.08;}
 double rs=radians*(1.-radians*radians/6.);
 double rc=1.-radians*radians/2.+radians*radians*radians*radians/24.;
 double ns=*sa*rc+*ca*rs,nc=*ca*rc-*sa*rs;
 double inv=1./SQRT(ns*ns+nc*nc);*sa=ns*inv;*ca=nc*inv;
}
static void draw_world(HDC dc){
 int xa,ya,xb,yb;double depth;
 /* Infinite laboratory checker hints. */
 for(int i=-8;i<=8;i++){
  V3 p1=v3(i*.35,0,-2.8),p2=v3(i*.35,0,2.8);
  project(p1,&xa,&ya,&depth);project(p2,&xb,&yb,&depth);
  stroke(dc,xa,ya,xb,yb,(i==0?CLR(48,91,99):CLR(29,49,64)),1);
  p1=v3(-2.8,0,i*.35);p2=v3(2.8,0,i*.35);
  project(p1,&xa,&ya,&depth);project(p2,&xb,&yb,&depth);
  stroke(dc,xa,ya,xb,yb,(i==0?CLR(48,91,99):CLR(29,49,64)),1);
 }
 /* Bridge lattice edges in projection. */
 for(int i=0;i<edge_count;i++){
  Edge *e=&edges[i];if(!e->alive)continue;
  project(nodes[e->a].p,&xa,&ya,&depth);project(nodes[e->b].p,&xb,&yb,&depth);
  DWORD col=e->damage>.45?CLR(249,101,94):
   (e->damage>.08?CLR(242,174,92):
   (e->original_rest>1e-12 && e->rest>e->original_rest*1.02?CLR(190,144,247):
   (nodes[e->a].t+nodes[e->b].t>630.?CLR(223,145,79):CLR(55,131,146))));
  stroke(dc,xa,ya,xb,yb,col,1);
 }
 /* Draw particles, large impact sphere is distinct from source lattice. */
 for(int i=0;i<node_count;i++){
  Node *p=&nodes[i];project(p->p,&xa,&ya,&depth);
  double s=zoom*560./(4.8+depth>1.6?4.8+depth:1.6);
  int r=roundi(p->r*s);if(r<2)r=2;if(r>38)r=38;
  DWORD col=p->kind==3?GOLD:(p->kind==1?CLR(124,213,161):(p->kind==2?CLR(188,168,231):ACCENT));
  if(p->t>325. && p->kind!=3)col=CLR(245,126,84);
  sphere(dc,xa,ya,r,col);
  if(r>=5)sphere(dc,xa-r/3,ya-r/3,r/4,CLR(222,248,244));
 }
}
static void energy_trace(HDC dc){
 int x=273,y=view_h-188,w=view_w-558,h=104;
 if(w<240||view_h<650)return;
 pill(dc,x,y,w,h,CLR(15,29,44));
 txt(dc,x+15,y+7,"MECHANICAL ENERGY",MUTED,f_small);
 txt(dc,x+w-119,y+7,"FRACTURE EVENTS",GOLD,f_small);
 if(trace_count<2)return;
 double lo=trace_mech[0],hi=trace_mech[0];
 for(int i=1;i<trace_count;i++){if(trace_mech[i]<lo)lo=trace_mech[i];if(trace_mech[i]>hi)hi=trace_mech[i];}
 double range=hi-lo;if(range<1.)range=1.;
 int gx=x+14,gw=w-28,gy=y+34,gh=55;
 for(int j=1;j<=3;j++)stroke(dc,gx,gy+j*gh/4,gx+gw,gy+j*gh/4,CLR(36,55,69),1);
 for(int i=1;i<trace_count;i++){
   int x1=gx+(i-1)*gw/(CHART_POINTS-1),x2=gx+i*gw/(CHART_POINTS-1);
   int y1=gy+gh-(int)((trace_mech[i-1]-lo)*gh/range),y2=gy+gh-(int)((trace_mech[i]-lo)*gh/range);
   stroke(dc,x1,y1,x2,y2,ACCENT,2);
   if(trace_fract[i]>trace_fract[i-1]){
     int f=trace_fract[i]-trace_fract[i-1];if(f>gh/7)f=gh/7;
     stroke(dc,x2,gy+gh,x2,gy+gh-6-f*5,GOLD,2);
   }
 }
}
static void button(HDC dc,int x,int y,int w,const char*key,const char*caption,DWORD color){
 pill(dc,x,y,w,43,PANEL2);rect(dc,x,y,3,43,color);
 txt(dc,x+13,y+6,key,color,f_label);txt(dc,x+42,y+11,caption,TEXT,f_small);
}
static void draw(HDC dc){
 RECT full={0,0,view_w,view_h};HBRUSH bg=CreateSolidBrush(UIBG);FillRect(dc,&full,bg);DeleteObject(bg);
 SetBkMode(dc,TRANSPARENT);
 // Main viewport and grid
 rect(dc,238,0,view_w-488,view_h,CLR(11,21,34));
 draw_world(dc);
 energy_trace(dc);
 // North HUD
 txt(dc,271,29,"MECHANICS / DETERMINISTIC NATIVE CORE",MUTED,f_small);
 txt(dc,271,49,"MECHANICS / EXPERIMENT LAB",TEXT,f_title);
 pill(dc,view_w-466,28,175,27,CLR(17,61,57));
 txt(dc,view_w-452,32,"ONE SHARED SOLVER",ACCENT,f_small);
 if(paused){pill(dc,view_w-466,28,175,27,CLR(72,49,41));txt(dc,view_w-452,32,"SIMULATION PAUSED",GOLD,f_small);}
 // Left sidebar
 rect(dc,0,0,239,view_h,PANEL);
 rect(dc,0,0,5,view_h,ACCENT);
 txt(dc,25,29,"AION / PRIMORDIA",ACCENT,f_title);
 txt(dc,26,63,"NATIVE RESEARCH LAB",MUTED,f_small);
 rect(dc,24,105,190,1,LINE_CLR);
 txt(dc,26,123,"MATTER EXPERIMENT",TEXT,f_label);
 txt(dc,26,149,"Plastic + damageable lattice",MUTED,f_small);
 txt(dc,26,171,"No recipes or tech tree",MUTED,f_small);
 txt(dc,26,203,"MATERIAL",MUTED,f_small);
 pill(dc,24,228,190,37,PANEL2);
 txt(dc,40,237,material_mode==0?"1  STONE-LIKE":(material_mode==1?"2  FIBROUS":"3  BRITTLE"),ACCENT,f_label);
 txt(dc,26,288,"PHYSICAL CONTROL",MUTED,f_small);
 button(dc,24,312,190,"I","IMPACT",GOLD);
 button(dc,24,367,190,"H","HEAT +25 K",CLR(245,126,84));
 button(dc,24,422,190,"T","TENSILE PULL",CLR(197,151,248));
 button(dc,24,477,190,"R","RESET WORLD",ACCENT);
 txt(dc,26,537,"CAMERA",MUTED,f_small);
 txt(dc,26,559,"Drag to orbit",TEXT,f_small);
 txt(dc,26,583,"Mouse wheel to zoom",TEXT,f_small);
 txt(dc,26,607,"Space: pause / resume",TEXT,f_small);
 txt(dc,26,631,"+ / -: time scale",TEXT,f_small);
 txt(dc,26,654,"S/L: checkpoints",ACCENT,f_small);
 txt(dc,26,678,"B: 9 tests -> CSV report",GOLD,f_small);
 rect(dc,24,view_h-94,191,1,LINE_CLR);
 txt(dc,26,view_h-76,"MODEL: coarse-grained",MUTED,f_small);
 txt(dc,26,view_h-57,"One kernel / C and Rust FFI",MUTED,f_small);
 txt(dc,26,view_h-38,"Windows x64 / 0.8 C reference",ACCENT,f_small);
 // Right panel
 rect(dc,view_w-249,0,249,view_h,PANEL);
 rect(dc,view_w-249,0,1,view_h,LINE_CLR);
 int rx=view_w-225;
 txt(dc,rx,34,"LIVE TELEMETRY",TEXT,f_title);
 rect(dc,rx,73,202,1,LINE_CLR);
 clearstr();integer(node_count);stat(dc,rx,93,"PARTICLES",sbuf,TEXT);
 clearstr();integer(alive_bonds());append(" / ");integer(edge_count);stat(dc,rx,152,"INTACT CONSTRAINTS",sbuf,ACCENT);
 clearstr();integer(fractures);stat(dc,rx,211,"FRACTURES",sbuf,GOLD);
 clearstr();integer(contacts);stat(dc,rx,270,"COLLISION IMPULSES",sbuf,TEXT);
 clearstr();fixed1(total_mass());append(" kg");stat(dc,rx,329,"TOTAL MASS",sbuf,TEXT);
 clearstr();fixed1(nodes[0].t);append(" K");stat(dc,rx,388,"SAMPLE TEMPERATURE",sbuf,CLR(242,146,109));
 clearstr();fixed1(sim_time);append(" s");stat(dc,rx,447,"SIMULATION TIME",sbuf,TEXT);
 clearstr();fixed1(clock_speed);append("x");stat(dc,rx,506,"TIME SCALE",sbuf,ACCENT);
 clearstr();fixed1(sim.max_strain*100.);append(" %");stat(dc,rx,565,"MAX SPRING STRAIN",sbuf,GOLD);
 clearstr();fixed1(aion_max_plastic_strain(&sim)*100.);append(" %");
 stat(dc,rx,619,"PERMANENT DEFORMATION",sbuf,CLR(197,151,248));
 rect(dc,rx,view_h-168,200,1,LINE_CLR);
 txt(dc,rx,view_h-148,"SOLVER + CHECKPOINT",MUTED,f_small);
 txt(dc,rx,view_h-127,"Semi-implicit Euler + sweep",TEXT,f_small);
 txt(dc,rx,view_h-105,"Component-aware contacts",TEXT,f_small);
 txt(dc,rx,view_h-83,"Plasticity + irreversible damage",TEXT,f_small);
 txt(dc,rx,view_h-61,"Fatigue + viscous heat",TEXT,f_small);
 txt(dc,rx,view_h-39,"B: repeatable material benchmark",ACCENT,f_small);
 // Floating bottom status
 pill(dc,view_w/2-215,view_h-68,430,38,CLR(21,35,52));
 txt(dc,view_w/2-193,view_h-59,checkpoint_status,TEXT,f_small);
}
static LRESULT __stdcall wndproc(HWND hwnd,UINT m,WPARAM w,LPARAM l){
 switch(m){
  case WM_DESTROY:KillTimer(hwnd,1);if(f_small)DeleteObject(f_small);if(f_label)DeleteObject(f_label);if(f_title)DeleteObject(f_title);if(f_huge)DeleteObject(f_huge);PostQuitMessage(0);return 0;
  case WM_SIZE:{RECT r;GetClientRect(hwnd,&r);view_w=r.right-r.left;view_h=r.bottom-r.top;if(view_w<780)view_w=780;if(view_h<540)view_h=540;InvalidateRect(hwnd,0,0);return 0;}
  case WM_TIMER:{if(!paused){int n=(int)(5*clock_speed);if(n>48)n=48;for(int i=0;i<n;i++)world_tick(.003);} tick_counter++; if(tick_counter%5==0)record_trace(); InvalidateRect(hwnd,0,0);return 0;}
  case WM_PAINT:{PAINTSTRUCT ps;HDC front=BeginPaint(hwnd,&ps);int w=view_w,h=view_h;
    HDC mem=CreateCompatibleDC(front);HBITMAP bmp=CreateCompatibleBitmap(front,w,h);
    HGDIOBJ old=SelectObject(mem,bmp);draw(mem);
    BitBlt(front,0,0,w,h,mem,0,0,SRCCOPY);
    SelectObject(mem,old);DeleteObject(bmp);DeleteDC(mem);EndPaint(hwnd,&ps);return 0;}
  case WM_KEYDOWN:{int k=(int)w;
    if(k=='R'){reset_world();trace_count=0;}if(k=='S')save_checkpoint();if(k=='L')load_checkpoint();if(k=='B')export_benchmark();if(k=='I')impulse();if(k=='H')heat_sample();if(k=='T')tensile_pull();
    if(k=='1'||k=='2'||k=='3'){material_mode=k-'1';reset_world();trace_count=0;}
    if(k==' ')paused=!paused;
    if(k==187||k==107){clock_speed=clock_speed<4?clock_speed*2:8.;}
    if(k==189||k==109){clock_speed=clock_speed>.25?clock_speed*.5:.25;}
    InvalidateRect(hwnd,0,0);return 0;}
  case WM_LBUTTONDOWN:{int x=(short)(l&0xffff),y=(short)((l>>16)&0xffff);
    if(x>=24&&x<214){if(y>=312&&y<355)impulse();else if(y>=367&&y<410)heat_sample();else if(y>=422&&y<465)tensile_pull();else if(y>=477&&y<520){reset_world();trace_count=0;}}
    else if(x>239&&x<view_w-249){mouse_drag=1;last_x=x;last_y=y;SetCapture(hwnd);}return 0;}
  case WM_MOUSEMOVE:{if(mouse_drag){int x=(short)(l&0xffff),y=(short)((l>>16)&0xffff);
    rot_pair(&yaw_s,&yaw_c,(x-last_x)*.005);
    rot_pair(&pitch_s,&pitch_c,(y-last_y)*.003);
    last_x=x;last_y=y;InvalidateRect(hwnd,0,0);
    }return 0;}
  case WM_LBUTTONUP:mouse_drag=0;ReleaseCapture();return 0;
  case WM_MOUSEWHEEL:{int delta=(short)((w>>16)&0xffff);
    if(delta>0&&zoom<2.5)zoom*=1.12;
    if(delta<0&&zoom>.35)zoom/=1.12;
    InvalidateRect(hwnd,0,0);return 0;}
 }
 return DefWindowProcA(hwnd,m,w,l);
}
void mainCRTStartup(void){
 reset_world();record_trace();HINSTANCE inst=GetModuleHandleA(0);
 WNDCLASSEXA wc={0};wc.cbSize=sizeof(wc);wc.lpfnWndProc=wndproc;wc.hInstance=inst;wc.hCursor=LoadCursorA(0,(const char*)32512);wc.lpszClassName="AION_CORE_NATIVE";
 if(!RegisterClassExA(&wc))ExitProcess(3);
 window=CreateWindowExA(0,wc.lpszClassName,"AION CORE 0.8 (C reference) | reproducible material experiments",WS_OVERLAPPEDWINDOW|WS_VISIBLE,60,60,1350,850,0,0,inst,0);
 if(!window)ExitProcess(4);
 f_small=CreateFontA(-14,0,0,0,400,0,0,0,0,0,0,0,0,"Segoe UI");
 f_label=CreateFontA(-16,0,0,0,650,0,0,0,0,0,0,0,0,"Segoe UI");
 f_title=CreateFontA(-21,0,0,0,700,0,0,0,0,0,0,0,0,"Segoe UI");
 f_huge=CreateFontA(-29,0,0,0,700,0,0,0,0,0,0,0,0,"Segoe UI");
 ShowWindow(window,SW_SHOW);UpdateWindow(window);SetTimer(window,1,16,0);
 MSG msg;while(GetMessageA(&msg,0,0,0)>0){TranslateMessage(&msg);DispatchMessageA(&msg);}
 ExitProcess(0);
}
