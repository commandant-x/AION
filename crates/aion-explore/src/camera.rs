//! Collision-following walking camera. Coordinates measured in metres.
use aion_world::World;
use glam::{Mat4,Vec3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub flying: bool,
}
impl Camera {
    pub fn new(world:&World)->Self {
        let x=256.0; let z=256.0;
        Self{position:Vec3::new(x,world.elevation_m(x as f64,z as f64) as f32+1.8,z),yaw:0.4,pitch:-0.08,flying:false}
    }
    pub fn look(&mut self,dx:f64,dy:f64) {
        self.yaw-=dx as f32*0.0022;
        self.pitch=(self.pitch-dy as f32*0.0022).clamp(-1.53,1.53);
    }
    pub fn forward(&self)->Vec3 {
        Vec3::new(self.yaw.sin()*self.pitch.cos(),self.pitch.sin(),-self.yaw.cos()*self.pitch.cos())
    }
    pub fn update(&mut self,world:&World,keys:&Keys,dt:f32) {
        let dt=dt.clamp(0.0,0.075);
        let forward=Vec3::new(self.yaw.sin(),0.0,-self.yaw.cos());
        let right=Vec3::new(-forward.z,0.0,forward.x);
        let mut motion=Vec3::ZERO;
        if keys.forward {motion+=forward};if keys.backward{motion-=forward};
        if keys.right{motion+=right};if keys.left{motion-=right};
        if self.flying {if keys.up{motion+=Vec3::Y};if keys.down{motion-=Vec3::Y};}
        let speed=if keys.sprint { 50.0 }else{12.0};
        if motion.length_squared()>0.0 {self.position+=motion.normalize()*speed*dt;}
        if !self.flying {
            let h=world.elevation_m(self.position.x as f64,self.position.z as f64) as f32+1.8;
            self.position.y=h;
        }
    }
    pub fn projection_view(&self,aspect:f32)->Mat4 {
        let view=Mat4::look_to_rh(self.position,self.forward(),Vec3::Y);
        Mat4::perspective_rh(70.0_f32.to_radians(),aspect.max(0.05),0.15,2500.0)*view
    }
}
#[derive(Default,Debug,Clone)]
pub struct Keys {
    pub forward:bool,pub backward:bool,pub left:bool,pub right:bool,
    pub sprint:bool,pub up:bool,pub down:bool,
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn walking_respects_terrain() {
        let w=World::new(7);let mut cam=Camera::new(&w);
        let mut keys=Keys::default();keys.forward=true;
        for _ in 0..500 {cam.update(&w,&keys,0.01)}
        let h=w.elevation_m(cam.position.x as f64,cam.position.z as f64) as f32;
        assert!((cam.position.y-h-1.8).abs()<0.01);
        assert!(cam.position.is_finite());
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