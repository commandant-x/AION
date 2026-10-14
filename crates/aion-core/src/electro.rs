//! Small-scale classical electrostatics with Plummer-style regularization.
//! Charges are specified in coulombs. This is NOT a quantum-chemistry or ionization model.
use crate::math::Vec3;
const K:f64=8_987_551_792.3; // N m^2 C^-2

#[derive(Clone,Debug)]
pub struct Charge {pub position:Vec3,pub velocity:Vec3,pub mass:f64,pub q:f64}
#[derive(Clone,Debug)]
pub struct ElectroSystem {pub particles:Vec<Charge>,pub softening:f64,pub time:f64}
impl ElectroSystem {
    pub fn new(softening:f64)->Self {assert!(softening>0.&&softening.is_finite());Self{particles:vec![],softening,time:0.}}
    pub fn add(&mut self,c:Charge) {assert!(c.position.finite()&&c.velocity.finite()&&c.mass>0.&&c.mass.is_finite()&&c.q.is_finite());self.particles.push(c);}
    fn forces(&self)->Vec<Vec3>{
        let n=self.particles.len();let mut result=vec![Vec3::ZERO;n];
        for i in 0..n {for j in i+1..n {
            let delta=self.particles[j].position-self.particles[i].position;
            let scale=-K*self.particles[i].q*self.particles[j].q/(delta.norm2()+self.softening.powi(2)).powf(1.5);
            let f=delta*scale;
            result[i]+=f;result[j]-=f;
        }}result
    }
    pub fn momentum(&self)->Vec3 {self.particles.iter().fold(Vec3::ZERO,|a,c|a+c.velocity*c.mass)}
    pub fn total_energy(&self)->f64 {
        let mut e:f64=self.particles.iter().map(|c|0.5*c.mass*c.velocity.norm2()).sum();
        for i in 0..self.particles.len(){for j in i+1..self.particles.len(){
            e+=K*self.particles[i].q*self.particles[j].q/
                ((self.particles[j].position-self.particles[i].position).norm2()+self.softening.powi(2)).sqrt();
        }}e
    }
    pub fn step(&mut self,dt:f64){
        assert!(dt>0.&&dt.is_finite()&&dt<=0.01);
        // Velocity Verlet, pair forces equal and opposite; adapt dt for strong near contacts.
        let f0=self.forces();
        for (p,f) in self.particles.iter_mut().zip(f0){
            p.velocity+=f*(dt/(2.*p.mass));p.position+=p.velocity*dt;
        }
        let f1=self.forces();
        for (p,f) in self.particles.iter_mut().zip(f1){p.velocity+=f*(dt/(2.*p.mass));}
        self.time+=dt;
    }
}
#[cfg(test)]mod tests {
    use super::*;
    fn pair(q1:f64,q2:f64)->ElectroSystem{
        let mut e=ElectroSystem::new(0.05);
        e.add(Charge{position:Vec3::new(-0.5,0.,0.),velocity:Vec3::ZERO,mass:0.01,q:q1});
        e.add(Charge{position:Vec3::new(0.5,0.,0.),velocity:Vec3::ZERO,mass:0.01,q:q2});e
    }
    #[test]fn like_charges_repulse(){let mut e=pair(1e-6,1e-6);e.step(0.001);assert!(e.particles[0].velocity.x<0.);assert!(e.particles[1].velocity.x>0.);}
    #[test]fn opposite_charges_attract(){let mut e=pair(1e-6,-1e-6);e.step(0.001);assert!(e.particles[0].velocity.x>0.);assert!(e.particles[1].velocity.x<0.);}
    #[test]fn equal_opposite_forces(){let mut e=pair(1e-6,-2e-6);let p=e.momentum();for _ in 0..100{e.step(0.0001);}assert!((e.momentum()-p).norm()<1e-12);}
    #[test]fn verlet_energy_approximately_constant(){let mut e=pair(1e-6,1e-6);let start=e.total_energy();for _ in 0..1000{e.step(0.0001);}assert!((e.total_energy()-start).abs()<1e-4);}
}
