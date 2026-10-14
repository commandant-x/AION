//! Rust adapter for the single AION 0.4 native reference physics engine.
//!
//! The Win32 viewer and this crate use the identical native/aion_sim.c code.
//! IMPORTANT: the solver is C pending validation of a Rust rewrite. Its
//! mechanics are coarse-grained research approximations, not atomic physics.
use std::{mem::MaybeUninit, os::raw::c_int};

pub const MAX_NODES: usize = 128;
pub const MAX_BONDS: usize = 1024;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Particle {
    pub pos: Vec3, pub vel: Vec3,
    pub radius: f64, pub mass: f64, pub temp: f64, pub heat_capacity: f64,
    pub material_id: c_int, pub component: c_int,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Spring {
    pub a: c_int, pub b: c_int, pub alive: c_int,
    pub rest: f64, pub stiffness: f64, pub threshold: f64, pub conductance: f64,
    pub original_rest: f64, pub yield_strain: f64, pub plastic_rate: f64,
    pub damage: f64, pub damage_onset: f64, pub damage_rate: f64, pub dashpot: f64,
}
#[repr(C)]
pub struct RawSim {
    pub nodes: [Particle; MAX_NODES], pub edges: [Spring; MAX_BONDS],
    pub node_count: c_int, pub edge_count: c_int, pub fractures: c_int,
    pub contacts: c_int, pub material_mode: c_int, pub rejected_steps: c_int,
    pub plastic_events: c_int, pub fatigue_fractures: c_int,
    pub sim_time: f64, pub gravity: f64, pub max_strain: f64,
    pub max_contact_impulse: f64,
    pub plastic_heat: f64, pub damage_heat: f64, pub damping_heat: f64, pub max_damage: f64,
    pub floor_enabled: c_int,
}
extern "C" {
    fn aion_reset(s: *mut RawSim, material_mode: c_int);
    fn aion_step(s: *mut RawSim, dt: f64);
    fn aion_impact(s: *mut RawSim);
    fn aion_heat(s: *mut RawSim);
    fn aion_pull(s: *mut RawSim);
    fn aion_max_plastic_strain(s: *const RawSim) -> f64;
    fn aion_mass(s: *const RawSim) -> f64;
    fn aion_total_energy(s: *const RawSim) -> f64;
    fn aion_checksum(s: *const RawSim) -> u64;
    fn aion_abi_size() -> u32;
}
/// Owns a native simulation. The object is fully initialised by aion_reset
/// before exposing any &RawSim reference (no Rust reads from uninitialised bytes).
pub struct Simulator { state: Box<RawSim> }
impl Simulator {
    pub fn new(material_mode: u8) -> Self {
        assert!(material_mode <= 2);
        assert_eq!(unsafe { aion_abi_size() } as usize, std::mem::size_of::<RawSim>(), "C/Rust ABI mismatch");
        let mut raw=Box::new(MaybeUninit::<RawSim>::uninit());
        unsafe {
            aion_reset(raw.as_mut_ptr(), material_mode.into());
            let state=Box::from_raw(Box::into_raw(raw) as *mut RawSim);
            Self{state}
        }
    }
    pub fn step(&mut self,dt:f64) {assert!(dt.is_finite()&&dt>0.&&dt<=0.1);unsafe{aion_step(&mut *self.state,dt)}}
    pub fn impact(&mut self){unsafe{aion_impact(&mut *self.state)}}
    pub fn pull(&mut self){unsafe{aion_pull(&mut *self.state)}}
    pub fn max_plastic_strain(&self)->f64{unsafe{aion_max_plastic_strain(&*self.state)}}
    pub fn heat_sample(&mut self){unsafe{aion_heat(&mut *self.state)}}
    pub fn mass(&self)->f64 {unsafe{aion_mass(&*self.state)}}
    pub fn total_energy(&self)->f64 {unsafe{aion_total_energy(&*self.state)}}
    pub fn checksum(&self)->u64 {unsafe{aion_checksum(&*self.state)}}
    pub fn state(&self)->&RawSim { &self.state }
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn same_engine_and_abi() {
        let sim=Simulator::new(0);
        assert_eq!(sim.state().node_count,73);
        assert_eq!(sim.state().edge_count,404);
        assert!((sim.mass()-97.4).abs()<1e-7);
    }
    #[test]fn replay_is_bitwise_stable() {
        let mut a=Simulator::new(0);let mut b=Simulator::new(0);
        for _ in 0..2000 {a.step(0.003);b.step(0.003)}
        assert_eq!(a.checksum(),b.checksum());
        assert!(a.state().fractures>0);
    }
    #[test]fn material_selection_and_heat(){
        let mut s=Simulator::new(1);
        assert_eq!(s.state().nodes[0].material_id,1);
        let energy=s.total_energy();s.heat_sample();
        assert!(s.total_energy()>energy);
        s.pull();
        s.step(0.003);
        assert!(s.checksum()!=0);
    }
}
