//! First-person locomotion: gravity, a deliberate jump, terrain and ocean contact.
//! All units are SI (metres and seconds). This is a kinematic character controller,
//! not the material solver, and swimming is currently represented by surface support.
use aion_world::{World, Earthworks, SEA_LEVEL_M};
use glam::{Mat4,Vec3};

const EYE_HEIGHT: f32 = 1.8;
const GRAVITY: f32 = 22.0;
const JUMP_SPEED: f32 = 8.0;
const MAX_STEP_GRADIENT: f32 = 1.4;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub flying: bool,
    pub grounded: bool,
    vertical_speed: f32,
}
impl Camera {
    pub fn new(world:&World)->Self {
        let (x,z)=find_coastal_spawn(world);
        let y=surface_eye_height(world,x,z);
        Self {position:Vec3::new(x,y,z),yaw:0.4,pitch:-0.08,flying:false,
              grounded:true,vertical_speed:0.0}
    }
    pub fn jump(&mut self) {
        if !self.flying && self.grounded {
            self.vertical_speed=JUMP_SPEED;
            self.grounded=false;
        }
    }
    pub fn toggle_flight(&mut self) {
        self.flying=!self.flying;
        self.vertical_speed=0.0;
        self.grounded=false;
    }
    pub fn look(&mut self,dx:f64,dy:f64) {
        self.yaw-=dx as f32*0.0022;
        self.pitch=(self.pitch-dy as f32*0.0022).clamp(-1.53,1.53);
    }
    pub fn forward(&self)->Vec3 {
        Vec3::new(self.yaw.sin()*self.pitch.cos(),self.pitch.sin(),-self.yaw.cos()*self.pitch.cos())
    }
    pub fn update(&mut self,world:&World,keys:&Keys,dt:f32) {
        self.update_with_edits(world,keys,dt,None);
    }
    pub fn update_with_edits(&mut self,world:&World,keys:&Keys,dt:f32,edits:Option<&Earthworks>) {
        let dt=dt.clamp(0.0,0.075);
        let forward=Vec3::new(self.yaw.sin(),0.0,-self.yaw.cos());
        let right=Vec3::new(-forward.z,0.0,forward.x);
        let mut motion=Vec3::ZERO;
        if keys.forward {motion+=forward;}if keys.backward{motion-=forward;}
        if keys.right{motion+=right;}if keys.left{motion-=right;}
        if self.flying {if keys.up{motion+=Vec3::Y;}if keys.down{motion-=Vec3::Y;}}
        let speed=if keys.sprint { 30.0 }else{9.0};
        if motion.length_squared()>0.0 && dt>0.0 {
            let candidate=self.position+motion.normalize()*speed*dt;
            let current_floor=surface_eye_height_edited(world,edits,self.position.x,self.position.z);
            let next_floor=surface_eye_height_edited(world,edits,candidate.x,candidate.z);
            let horizontal=((candidate.x-self.position.x).powi(2)+(candidate.z-self.position.z).powi(2)).sqrt();
            if self.flying || !self.grounded || next_floor-current_floor <=
                MAX_STEP_GRADIENT*horizontal+0.2 {
                self.position=candidate;
            }
        }
        if !self.flying {
            self.vertical_speed-=GRAVITY*dt;
            self.position.y+=self.vertical_speed*dt;
            let floor=surface_eye_height_edited(world,edits,self.position.x,self.position.z);
            if self.position.y<=floor {
                self.position.y=floor;
                self.vertical_speed=0.0;
                self.grounded=true;
            }else{self.grounded=false;}
        }
    }
    pub fn projection_view(&self,aspect:f32)->Mat4 {
        let view=Mat4::look_to_rh(self.position,self.forward(),Vec3::Y);
        Mat4::perspective_rh(70.0_f32.to_radians(),aspect.max(0.05),0.15,2800.0)*view
    }
}
fn surface_eye_height(world:&World,x:f32,z:f32)->f32 {
    // The ocean is a visible, collidable support surface (real buoyancy remains TODO).
    (world.elevation_m(x as f64,z as f64) as f32).max(SEA_LEVEL_M)+EYE_HEIGHT
}
fn surface_eye_height_edited(world:&World,edits:Option<&Earthworks>,x:f32,z:f32)->f32 {
    let raw=match edits {Some(e)=>e.elevation_m(world,x as f64,z as f64) as f32,
                         None=>world.elevation_m(x as f64,z as f64) as f32};
    raw.max(SEA_LEVEL_M)+EYE_HEIGHT
}
fn find_coastal_spawn(world:&World)->(f32,f32) {
    // Search a fixed deterministic patch for an accessible coast. Both directions
    // and negative coordinates are allowed. Fallback is valid for land-only seeds.
    let mut best=(256.0f32,256.0f32);
    let mut score=f32::MAX;
    for iz in -10..=10 {for ix in -10..=10 {
        let x=ix as f32*128.0;
        let z=iz as f32*128.0;
        let h=world.elevation_m(x as f64,z as f64) as f32;
        let sea_near=[(-128.,0.),(128.,0.),(0.,-128.),(0.,128.)].iter()
            .any(|&(dx,dz)| world.elevation_m((x+dx) as f64,(z+dz) as f64)<SEA_LEVEL_M as f64);
        if !sea_near || !(2.0..=45.0).contains(&h) {continue;}
        let s=(x*x+z*z).sqrt()+h*2.0;
        if s<score {score=s;best=(x,z);}
    }}
    best
}
#[derive(Default,Debug,Clone)]
pub struct Keys {
    pub forward:bool,pub backward:bool,pub left:bool,pub right:bool,
    pub sprint:bool,pub up:bool,pub down:bool,
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn walking_respects_terrain_and_ocean_surface() {
        let w=World::new(7);let mut cam=Camera::new(&w);
        let mut keys=Keys::default();keys.forward=true;
        for _ in 0..500 {cam.update(&w,&keys,0.01)}
        let expected=surface_eye_height(&w,cam.position.x,cam.position.z);
        assert!((cam.position.y-expected).abs()<0.03);
        assert!(cam.position.is_finite());
    }
    #[test] fn jump_rises_falls_and_lands_without_repeat() {
        let w=World::new(91);let mut cam=Camera::new(&w);
        let initial=cam.position.y;
        cam.jump();assert!(!cam.grounded);
        cam.update(&w,&Keys::default(),0.04);
        assert!(cam.position.y>initial+0.1);
        for _ in 0..120 {cam.update(&w,&Keys::default(),0.02);}
        assert!(cam.grounded);
        assert!((cam.position.y-initial).abs()<0.01);
        let before=cam.position.y;
        cam.update(&w,&Keys::default(),0.01);
        assert!((cam.position.y-before).abs()<0.001);
    }
    #[test] fn flight_allows_vertical_motion() {
        let w=World::new(13);let mut cam=Camera::new(&w);
        let y=cam.position.y;
        cam.toggle_flight();let keys=Keys{up:true,..Default::default()};
        cam.update(&w,&keys,0.05);
        assert!(cam.position.y>y);
    }
    #[test] fn dt_is_bounded_and_pitch_clamped() {
        let w=World::new(0);let mut cam=Camera::new(&w);
        cam.look(0.,100000.);assert!(cam.pitch>=-1.53);
        cam.look(0.,-200000.);assert!(cam.pitch<=1.53);
        let p=cam.position;
        let k=Keys{forward:true,..Default::default()};
        cam.update(&w,&k,100000.0);
        assert!((cam.position-p).length()<5.0);
    }
    #[test] fn matrices_have_no_nan() {
        let w=World::new(0);let cam=Camera::new(&w);
        assert!(cam.projection_view(1920.0/1080.0).to_cols_array().iter().all(|f|f.is_finite()));
    }
}