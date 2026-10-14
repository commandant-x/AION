//! Native Win32 diagnostic visualizer using the pure Rust AION mechanics solver.
//! This is a GDI-rendered 3D projection, NOT the planned GPU renderer.
//! Zero non-Rust runtime dependencies. Windows window creation is smoke-tested in CI.
#![allow(non_snake_case)]
use aion_rust_sim::{AionSim, V3};
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::{Mutex, OnceLock};

// 64-bit Win32 ABI. The public Rust crate is intentionally dependency-free.
type H = isize;
const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_SIZE: u32 = 0x0005;
const WM_PAINT: u32 = 0x000F;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_KEYDOWN: u32 = 0x0100;
const WM_TIMER: u32 = 0x0113;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_MOUSEWHEEL: u32 = 0x020A;
const WM_SETCURSOR: u32 = 0x0020;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_VISIBLE: u32 = 0x10000000;
const SW_SHOW: i32 = 5;
const TRANSPARENT: i32 = 1;
const PS_SOLID: i32 = 0;
const NULL_PEN: i32 = 8;
const SRCCOPY: u32 = 0x00CC0020;
const FW_NORMAL: i32 = 400;
const FW_BOLD: i32 = 700;
const COLOR_WINDOW: usize = 5;
const IDC_ARROW: usize = 32512;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect { left: i32, top: i32, right: i32, bottom: i32 }
impl Rect {
    fn from_xywh(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { left: x, top: y, right: x + w, bottom: y + h }
    }
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}
#[repr(C)]
struct Point { x: i32, y: i32 }
#[repr(C)]
struct Msg { hwnd: H, message: u32, wparam: usize, lparam: isize, time: u32, pt: Point, private_: u32 }
#[repr(C)]
struct WndClass {
    style: u32,
    wndproc: Option<unsafe extern "system" fn(H, u32, usize, isize) -> isize>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: H,
    icon: H,
    cursor: H,
    background: H,
    menu: *const u16,
    name: *const u16,
}
#[repr(C)]
struct PaintStruct {
    dc: H,
    erase: i32,
    paint: Rect,
    restore: i32,
    inc_update: i32,
    reserved: [u8; 32],
}

#[link(name="user32")]
extern "system" {
    fn RegisterClassW(wc: *const WndClass) -> u16;
    fn CreateWindowExW(ex:u32, class:*const u16, title:*const u16, style:u32,
      x:i32,y:i32,w:i32,h:i32,parent:H,menu:H,inst:H,param:*const c_void)->H;
    fn DefWindowProcW(hwnd:H,msg:u32,wparam:usize,lparam:isize)->isize;
    fn DestroyWindow(hwnd:H)->i32;
    fn ShowWindow(hwnd:H,cmd:i32)->i32;
    fn UpdateWindow(hwnd:H)->i32;
    fn GetMessageW(msg:*mut Msg,hwnd:H,min:u32,max:u32)->i32;
    fn TranslateMessage(msg:*const Msg)->i32;
    fn DispatchMessageW(msg:*const Msg)->isize;
    fn PostQuitMessage(code:i32);
    fn InvalidateRect(hwnd:H,rect:*const Rect,erase:i32)->i32;
    fn BeginPaint(hwnd:H,ps:*mut PaintStruct)->H;
    fn EndPaint(hwnd:H,ps:*const PaintStruct)->i32;
    fn GetClientRect(hwnd:H,rect:*mut Rect)->i32;
    fn SetTimer(hwnd:H,id:usize,interval:u32,proc:usize)->usize;
    fn KillTimer(hwnd:H,id:usize)->i32;
    fn LoadCursorW(inst:H,name:*const u16)->H;
    fn MessageBoxW(hwnd:H,text:*const u16,title:*const u16,typ:u32)->i32;
    fn GetDC(hwnd:H)->H;
    fn ReleaseDC(hwnd:H,dc:H)->i32;
}
#[link(name="kernel32")]
extern "system" {
    fn GetModuleHandleW(name:*const u16)->H;
}
#[link(name="gdi32")]
extern "system" {
    fn CreateCompatibleDC(dc:H)->H;
    fn DeleteDC(dc:H)->i32;
    fn CreateCompatibleBitmap(dc:H,w:i32,h:i32)->H;
    fn SelectObject(dc:H,obj:H)->H;
    fn DeleteObject(obj:H)->i32;
    fn CreateSolidBrush(c:u32)->H;
    fn CreatePen(style:i32,width:i32,color:u32)->H;
    fn GetStockObject(id:i32)->H;
    fn Rectangle(dc:H,x:i32,y:i32,r:i32,b:i32)->i32;
    fn RoundRect(dc:H,x:i32,y:i32,r:i32,b:i32,ew:i32,eh:i32)->i32;
    fn Ellipse(dc:H,x:i32,y:i32,r:i32,b:i32)->i32;
    fn MoveToEx(dc:H,x:i32,y:i32,previous:*mut Point)->i32;
    fn LineTo(dc:H,x:i32,y:i32)->i32;
    fn BitBlt(dst:H,x:i32,y:i32,w:i32,h:i32,src:H,sx:i32,sy:i32,rop:u32)->i32;
    fn SetBkMode(dc:H,mode:i32)->i32;
    fn SetTextColor(dc:H,c:u32)->u32;
    fn TextOutW(dc:H,x:i32,y:i32,text:*const u16,len:i32)->i32;
    fn CreateFontW(h:i32,w:i32,esc:i32,orient:i32,weight:i32,italic:u32,
      underline:u32,strike:u32,charset:u32,out_precision:u32,clip_precision:u32,
      quality:u32,pitch:u32,face:*const u16)->H;
}
#[link(name="user32")]
extern "system" {
    fn FillRect(dc:H,rect:*const Rect,brush:H)->i32;
}

fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(std::iter::once(0)).collect()}
fn color(r:u8,g:u8,b:u8)->u32{r as u32 | ((g as u32)<<8) | ((b as u32)<<16)}
const BG:u32=0x17100B; // BGR: #0B1017
const PANEL:u32=0x251C14;
const TEXT:u32=0xECEBE8;
const MUTED:u32=0xA4998E;
const TEAL:u32=0xA5D9A0;
const ORANGE:u32=0x638FEA;

fn fill(dc:H,r:Rect,c:u32){unsafe{let b=CreateSolidBrush(c);if b!=0{FillRect(dc,&r,b);DeleteObject(b);}}}
fn line(dc:H,x:i32,y:i32,ex:i32,ey:i32,c:u32,width:i32){
 unsafe{let p=CreatePen(PS_SOLID,width.max(1),c);if p!=0{let old=SelectObject(dc,p);MoveToEx(dc,x,y,null_mut());LineTo(dc,ex,ey);SelectObject(dc,old);DeleteObject(p);}}
}
fn sphere(dc:H,x:i32,y:i32,r:i32,c:u32){
 unsafe{let b=CreateSolidBrush(c);if b!=0{let old_b=SelectObject(dc,b);let old_p=SelectObject(dc,GetStockObject(NULL_PEN));Ellipse(dc,x-r,y-r,x+r,y+r);SelectObject(dc,old_p);SelectObject(dc,old_b);DeleteObject(b);}}
}
fn rounded(dc:H,r:Rect,c:u32,rad:i32){
 unsafe{let b=CreateSolidBrush(c);if b!=0{let old_b=SelectObject(dc,b);let old_p=SelectObject(dc,GetStockObject(NULL_PEN));RoundRect(dc,r.left,r.top,r.right,r.bottom,rad,rad);SelectObject(dc,old_p);SelectObject(dc,old_b);DeleteObject(b);}}
}
fn text(dc:H,x:i32,y:i32,s:&str,c:u32,size:i32,bold:bool){
 unsafe{
   let face=wide("Segoe UI");let font=CreateFontW(-size,0,0,0,if bold{FW_BOLD}else{FW_NORMAL},0,0,0,1,0,0,5,0,face.as_ptr());
   if font!=0 {let previous=SelectObject(dc,font);SetTextColor(dc,c);SetBkMode(dc,TRANSPARENT);
   let chars:Vec<u16>=s.encode_utf16().collect();TextOutW(dc,x,y,chars.as_ptr(),chars.len() as i32);
   SelectObject(dc,previous);DeleteObject(font);}
 }
}

struct App {
    sim:AionSim,paused:bool,yaw:f64,pitch:f64,zoom:f64,dragging:bool,last:(i32,i32),
    material:i32,frame:u64,event:String,status:String,
}
impl App {
 fn new()->Self{ Self{sim:AionSim::laboratory(0),paused:false,yaw:-0.60,pitch:0.33,zoom:1.0,
   dragging:false,last:(0,0),material:0,frame:0,event:"Simulation initialisée".into(),status:"RUNNING".into()} }
 fn tick(&mut self){
  if self.paused{return}
  for _ in 0..3{self.sim.step(1.0/120.0)}
  self.frame+=1;
  if !self.sim.sim_time.is_finite(){self.paused=true;self.status="NUMERICAL ERROR".into();}
 }
 fn action(&mut self,a:usize){match a{
  0=>{self.sim.impact();self.event="Nouvel impact applique".into();},
  1=>{self.sim.pull();self.event="Traction appliquee".into();},
  2=>{self.sim.heat();self.event="Chaleur injectee".into();},
  3=>{self.paused=!self.paused;self.event=(if self.paused{"Simulation en pause"}else{"Simulation relancee"}).into();},
  4=>{self.sim.reset(self.material);self.event="Monde reinitialise".into();self.paused=false;},
  5=>{let out=self.sim.encode_v7();if let Some(data)=out{let path=checkpoint_path();match std::fs::write(&path,data){Ok(())=>self.event=format!("Sauvegarde : {}",path.display()),Err(e)=>self.event=format!("Sauvegarde impossible : {e}")}}else{self.event="Sauvegarde refusee : etat invalide".into()}},
  6=>{let path=checkpoint_path();match std::fs::read(&path).ok().and_then(|x|AionSim::decode_v7(&x)) {Some(v)=>{self.material=v.material_mode;self.sim=v;self.event="Etat AION7 restaure".into();},None=>self.event="Aucune sauvegarde AION7 valide".into()}},
  _=>{}}
 }
 fn material(&mut self,mode:i32){self.material=mode;self.sim.reset(mode);self.event=format!("Materiau {} charge",["MINERAL","FIBRE","FRAGILE"][mode as usize]);self.paused=false;}
}
fn checkpoint_path()->std::path::PathBuf{
 std::env::current_exe().ok().and_then(|p|p.parent().map(|v|v.join("aion_checkpoint.aion7"))).unwrap_or_else(||std::path::PathBuf::from("aion_checkpoint.aion7"))
}
static APP:OnceLock<Mutex<App>>=OnceLock::new();
fn app()->&'static Mutex<App>{APP.get_or_init(||Mutex::new(App::new()))}

fn button_rect(w:i32,idx:usize)->Rect{
 let x=w-335;let gap=9;let bh=42;let bw=153;
 Rect::from_xywh(x+(idx%2) as i32*(bw+gap),516+(idx/2) as i32*(bh+gap),bw,bh)
}
fn select_material(app:&mut App,w:i32,x:i32,y:i32)->bool{
 let xs=w-335;
 if (432..476).contains(&y) && (xs..xs+315).contains(&x){
  let index=((x-xs)*3/315).clamp(0,2);
  app.material(index);return true;
 }
 false
}

fn project(p:V3,cx:i32,cy:i32,scale:f64,yaw:f64,pitch:f64)->(i32,i32,f64){
 let (sy,cyaw)=yaw.sin_cos();let (sp,cp)=pitch.sin_cos();
 let u=p.x*cyaw+p.z*sy;let depth=-p.x*sy+p.z*cyaw;
 let v=(p.y-1.5)*cp-depth*sp;
 let depth_scale=(1.0+depth*0.13).clamp(0.65,1.45);
 ((cx as f64+u*scale/depth_scale) as i32,(cy as f64-v*scale/depth_scale) as i32,depth)
}
fn draw_world(dc:H,app:&App,rect:Rect){
 let w=rect.right-rect.left;let h=rect.bottom-rect.top;
 let cx=rect.left+w/2;let cy=rect.top+h/2+25;let scale=(w.min(h) as f64)*0.47*app.zoom;
 // Perspective floor grid, 3D-projected from world coordinates.
 for i in -9..=9 {
  let f=i as f64*0.3;
  let (a,b,_)=project(V3::new(f,0.,-2.7),cx,cy,scale,app.yaw,app.pitch);
  let (c,d,_)=project(V3::new(f,0.,2.7),cx,cy,scale,app.yaw,app.pitch);
  line(dc,a,b,c,d,color(29,47,60),1);
  let (a,b,_)=project(V3::new(-2.7,0.,f),cx,cy,scale,app.yaw,app.pitch);
  let (c,d,_)=project(V3::new(2.7,0.,f),cx,cy,scale,app.yaw,app.pitch);
  line(dc,a,b,c,d,color(29,47,60),1);
 }
 // Only bonds physically alive are drawn. Color encodes irreversible damage.
 for e in app.sim.edges.iter().take(app.sim.edge_count.max(0) as usize) {
  if e.alive==0 || e.a<0 || e.b<0 {continue}
  let a=app.sim.nodes[e.a as usize].p;let b=app.sim.nodes[e.b as usize].p;
  let (x,y,_)=project(a,cx,cy,scale,app.yaw,app.pitch);
  let (x2,y2,_)=project(b,cx,cy,scale,app.yaw,app.pitch);
  let k=e.damage.clamp(0.,1.);
  let c=if k>.6{color(240,122,69)}else if k>.13{color(241,183,103)}else{color(80,143,164)};
  line(dc,x,y,x2,y2,c,if k>.5{2}else{1});
 }
 let mut nodes:Vec<_>=app.sim.nodes.iter().take(app.sim.node_count.max(0) as usize).collect();
 nodes.sort_by(|a,b|{
  let za=-a.p.x*app.yaw.sin()+a.p.z*app.yaw.cos();
  let zb=-b.p.x*app.yaw.sin()+b.p.z*app.yaw.cos();
  zb.total_cmp(&za)
 });
 for n in nodes {
  let (x,y,_)=project(n.p,cx,cy,scale,app.yaw,app.pitch);
  let radius=((n.r*scale).clamp(3.,24.)) as i32;
  let c=if n.kind==3{color(220,147,253)}else if n.t>350.{color(255,145,77)}else{
    match n.kind {1=>color(125,225,190),2=>color(242,168,137),_=>color(191,214,219)}
  };
  sphere(dc,x,y,radius+2,color(20,32,47));sphere(dc,x,y,radius,c);
  sphere(dc,x-radius/4,y-radius/3,(radius/4).max(1),color(250,252,255));
 }
 rounded(dc,Rect::from_xywh(rect.left+16,rect.top+16,170,35),color(18,36,49),12);
 text(dc,rect.left+28,rect.top+24,"LIVE SIMULATION",TEAL,13,true);
 text(dc,rect.left+19,rect.bottom-42,"DRAG : CAMERA     MOLETTE : ZOOM",color(153,179,192),12,false);
}
fn draw(dc:H,w:i32,h:i32,app:&App){
 let w=w.max(760);let h=h.max(590);
 fill(dc,Rect::from_xywh(0,0,w,h),BG);
 fill(dc,Rect::from_xywh(0,0,w,78),color(13,22,32));
 fill(dc,Rect::from_xywh(0,76,w,2),color(40,74,87));
 sphere(dc,30,37,8,color(112,227,187));
 text(dc,52,16,"AION / PRIMORDIA",TEXT,26,true);
 text(dc,54,49,"CORE 0.10  //  RUST PHYSICS LAB",MUTED,12,false);
 rounded(dc,Rect::from_xywh(w-220,24,195,32),color(27,55,61),12);
 text(dc,w-205,31,"NATIVE WINDOWS  x64",TEAL,12,true);
 let view=Rect::from_xywh(18,96,w-389,h-122);
 rounded(dc,view,color(17,31,42),17);
 draw_world(dc,app,Rect::from_xywh(view.left+5,view.top+5,view.right-view.left-10,view.bottom-view.top-10));
 let right=Rect::from_xywh(w-352,96,335,h-122);
 rounded(dc,right,PANEL,14);
 let x=right.left+17;
 text(dc,x,114,"PHYSICS TELEMETRY",TEXT,17,true);
 fill(dc,Rect::from_xywh(x,149,299,1),color(55,68,77));
 text(dc,x,163,"STATUS",MUTED,11,true);
 text(dc,x+122,160,if app.paused{"PAUSED"}else{"RUNNING"},if app.paused{ORANGE}else{TEAL},15,true);
 text(dc,x,201,"ELAPSED",MUTED,11,true);
 text(dc,x+122,198,&format!("{:.2} s",app.sim.sim_time),TEXT,16,true);
 text(dc,x,239,"NODES",MUTED,11,true);
 text(dc,x+122,236,&format!("{}",app.sim.node_count),TEXT,16,true);
 text(dc,x,277,"ACTIVE BONDS",MUTED,11,true);
 text(dc,x+122,274,&format!("{} / {}",app.sim.alive_bonds(),app.sim.edge_count),TEXT,16,true);
 text(dc,x,315,"FRACTURES",MUTED,11,true);
 text(dc,x+122,312,&format!("{}",app.sim.fractures),color(245,154,102),16,true);
 text(dc,x,353,"MAX DAMAGE",MUTED,11,true);
 text(dc,x+122,350,&format!("{:.1}%",app.sim.max_damage*100.),TEXT,16,true);
 fill(dc,Rect::from_xywh(x,390,299,1),color(55,68,77));
 text(dc,x,402,"MATERIAL EXPERIMENT",TEXT,13,true);
 let materials=["STONE","FIBRE","BRITTLE"];
 for (i,label) in materials.iter().enumerate(){
   let xx=x+i as i32*105;let c=if app.material==i as i32{color(45,90,92)}else{color(34,46,55)};
   rounded(dc,Rect::from_xywh(xx,432,100,43),c,10);
   text(dc,xx+13,446,label,if app.material==i as i32{TEAL}else{MUTED},11,true);
 }
 text(dc,x,491,"EXPERIMENT CONTROLS",TEXT,13,true);
 let labels=["IMPACT  [I]","PULL  [T]","HEAT  [H]","PAUSE  [SPACE]","RESET  [R]","SAVE  [S]","LOAD  [L]"];
 for (i,label) in labels.iter().enumerate(){
    let r=button_rect(w,i);rounded(dc,r,if i==0{color(45,91,90)}else{color(37,53,65)},11);
    text(dc,r.left+12,r.top+13,label,if i==0{TEAL}else{TEXT},12,true);
 }
 let bottom=516+4*51+4;
 if bottom < h-60{fill(dc,Rect::from_xywh(x,bottom,299,1),color(55,68,77));
   text(dc,x,bottom+9,"LAST ACTION",MUTED,11,true);
   let trimmed: String=app.event.chars().take(45).collect();text(dc,x,bottom+29,&trimmed,TEXT,11,false);
 }
 text(dc,20,h-20,"PHYSICS: RUST  |  RENDER: WIN32/GDI  |  SIMULATION: 120 Hz",MUTED,11,false);
}

unsafe extern "system" fn wndproc(hwnd:H,msg:u32,wparam:usize,lparam:isize)->isize{
 match msg{
 WM_CREATE=>{SetTimer(hwnd,1,33,0);0}
 WM_ERASEBKGND=>1,
 WM_TIMER=>{if wparam==1{let mut a=app().lock().unwrap();a.tick();drop(a);InvalidateRect(hwnd,null(),0);}0},
 WM_PAINT=>{
   let mut ps=PaintStruct{dc:0,erase:0,paint:Rect::default(),restore:0,inc_update:0,reserved:[0;32]};
   let hdc=BeginPaint(hwnd,&mut ps);if hdc==0{return 0}
   let mut rc=Rect::default();GetClientRect(hwnd,&mut rc);let (w,h)=(rc.right.max(1),rc.bottom.max(1));
   let off=CreateCompatibleDC(hdc);
   if off!=0{let bmp=CreateCompatibleBitmap(hdc,w,h);
     if bmp!=0{let old=SelectObject(off,bmp);let a=app().lock().unwrap();draw(off,w,h,&a);drop(a);
       BitBlt(hdc,0,0,w,h,off,0,0,SRCCOPY);SelectObject(off,old);DeleteObject(bmp);}
     DeleteDC(off);
   }
   EndPaint(hwnd,&ps);0
 },
 WM_KEYDOWN=>{
   let mut a=app().lock().unwrap();match wparam as u32{
     0x49=>a.action(0),0x54=>a.action(1),0x48=>a.action(2),0x20=>a.action(3),
     0x52=>a.action(4),0x53=>a.action(5),0x4C=>a.action(6),
     0x31=>a.material(0),0x32=>a.material(1),0x33=>a.material(2),
     0x25=>a.yaw-=0.12,0x27=>a.yaw+=0.12,0x26=>a.pitch=(a.pitch+.09).min(1.25),
     0x28=>a.pitch=(a.pitch-.09).max(-1.2),0x1B=>{drop(a);DestroyWindow(hwnd);return 0},_=>{}
   }drop(a);InvalidateRect(hwnd,null(),0);0
 },
 WM_LBUTTONDOWN=>{
   let x=lparam as i16 as i32;let y=(lparam>>16) as i16 as i32;
   let mut rc=Rect::default();GetClientRect(hwnd,&mut rc);let w=rc.right;
   let mut a=app().lock().unwrap();
   if select_material(&mut a,w,x,y){}
   else if x>=w-352 {for i in 0..7{if button_rect(w,i).contains(x,y){a.action(i);break}}}
   else{a.dragging=true;a.last=(x,y)}
   drop(a);InvalidateRect(hwnd,null(),0);0
 },
 WM_LBUTTONUP=>{app().lock().unwrap().dragging=false;0},
 WM_MOUSEMOVE=>{let x=lparam as i16 as i32;let y=(lparam>>16) as i16 as i32;
   let mut a=app().lock().unwrap();if a.dragging{a.yaw+=(x-a.last.0) as f64*.009;a.pitch=(a.pitch+(y-a.last.1) as f64*.006).clamp(-1.2,1.2);a.last=(x,y);drop(a);InvalidateRect(hwnd,null(),0)}0
 },
 WM_MOUSEWHEEL=>{let delta=((wparam>>16) as i16) as i32;let mut a=app().lock().unwrap();a.zoom=(a.zoom*if delta>0{1.10}else{0.90}).clamp(.30,3.0);drop(a);InvalidateRect(hwnd,null(),0);0},
 WM_SIZE=>{InvalidateRect(hwnd,null(),0);0},
 WM_DESTROY=>{KillTimer(hwnd,1);PostQuitMessage(0);0},
 WM_SETCURSOR=>DefWindowProcW(hwnd,msg,wparam,lparam),
 _=>DefWindowProcW(hwnd,msg,wparam,lparam),
 }
}

pub fn run(smoke:bool){
 unsafe{
   let name=wide("AionPrimordiaRustWindow");let title=wide("AION | PRIMORDIA CORE — Rust Physics Lab");
   let inst=GetModuleHandleW(null());
   let wc=WndClass{style:0,wndproc:Some(wndproc),cls_extra:0,wnd_extra:0,instance:inst,icon:0,
       cursor:LoadCursorW(0,IDC_ARROW as *const u16),background:(COLOR_WINDOW+1) as H,
       menu:null(),name:name.as_ptr()};
   if RegisterClassW(&wc)==0{MessageBoxW(0,wide("Unable to register native window class").as_ptr(),title.as_ptr(),0x10);std::process::exit(2)}
   let style=WS_OVERLAPPEDWINDOW | if smoke{0}else{WS_VISIBLE};
   let hwnd=CreateWindowExW(0,name.as_ptr(),title.as_ptr(),style,160,90,1420,900,0,0,inst,null());
   if hwnd==0{MessageBoxW(0,wide("Unable to create native application window").as_ptr(),title.as_ptr(),0x10);std::process::exit(3)}
   if smoke{
     let mut rect=Rect::default();assert_ne!(GetClientRect(hwnd,&mut rect),0);assert!(rect.right>0&&rect.bottom>0);
     // Exercise the rendering code while hidden, on the actual Windows graphics subsystem.
     let hdc=GetDC(hwnd);
     if hdc!=0{let mut a=app().lock().unwrap();a.tick();drop(a);
       let compatible=CreateCompatibleDC(hdc);assert_ne!(compatible,0);
       let bitmap=CreateCompatibleBitmap(hdc,rect.right,rect.bottom);assert_ne!(bitmap,0);
       let old=SelectObject(compatible,bitmap);draw(compatible,rect.right,rect.bottom,&app().lock().unwrap());
       SelectObject(compatible,old);DeleteObject(bitmap);DeleteDC(compatible);
       ReleaseDC(hwnd,hdc);
     }
     DestroyWindow(hwnd);return
   }
   ShowWindow(hwnd,SW_SHOW);UpdateWindow(hwnd);
   let mut m=Msg{hwnd:0,message:0,wparam:0,lparam:0,time:0,pt:Point{x:0,y:0},private_:0};
   loop {let res=GetMessageW(&mut m,0,0,0);if res<=0{break}TranslateMessage(&m);DispatchMessageW(&m);}
 }
}