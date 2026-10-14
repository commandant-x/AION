//! Mechanically coupled, non-recipe material voxel lattice experiment.
//! This is a mass-spring approximation; it is NOT finite-element fracture mechanics.
use crate::math::Vec3;
use crate::physics::{World, Particle, Material, Bond};

#[derive(Clone, Copy, Debug)]
pub struct LatticeSpec {
    pub nx:usize, pub ny:usize, pub nz:usize,
    pub pitch:f64, pub particle_radius:f64,
    pub stiffness:f64, pub break_force:f64, pub heat_conductance:f64,
    pub diagonals:bool,
}
impl LatticeSpec {
    pub fn experimental_block()->Self {Self {
        nx:5,ny:4,nz:3,pitch:.22,particle_radius:.065,
        stiffness:900., break_force:55., heat_conductance:1.8, diagonals:true,
    }}
    pub fn validate(&self)->Result<(),&'static str> {
        if self.nx==0 || self.ny==0 || self.nz==0 || self.nx>100 || self.ny>100 || self.nz>100 {return Err("invalid grid dimensions");}
        if self.nx.saturating_mul(self.ny).saturating_mul(self.nz)>10_000 {return Err("too many particles");}
        if !self.pitch.is_finite() || !self.particle_radius.is_finite() || self.pitch<=2.*self.particle_radius || self.particle_radius<=0. {return Err("overlapping particles");}
        if !self.stiffness.is_finite() || self.stiffness<=0. || !self.break_force.is_finite() || self.break_force<=0. || !self.heat_conductance.is_finite() || self.heat_conductance<0. {return Err("invalid material parameters");}
        Ok(())
    }
}

/// Add a material lattice to `world`; return the particle indices.
pub fn add_lattice(world:&mut World, spec:LatticeSpec, origin:Vec3, material:Material, temp:f64)->Result<Vec<usize>,&'static str>{
    spec.validate()?;
    if !origin.finite() || !material.validate() || !temp.is_finite() || temp<0. {return Err("invalid state");}
    let mut ids = Vec::with_capacity(spec.nx*spec.ny*spec.nz);
    for z in 0..spec.nz {for y in 0..spec.ny {for x in 0..spec.nx {
        ids.push(world.add(Particle::sphere(origin+Vec3::new(x as f64 * spec.pitch, y as f64 * spec.pitch, z as f64 * spec.pitch), spec.particle_radius, material, temp)));
    }}}
    let index=|x:usize,y:usize,z:usize| x+spec.nx*(y+spec.ny*z);
    let mut link = |a:usize,b:usize,rest:f64| {
        world.bond(Bond::new(ids[a],ids[b],rest,spec.stiffness,spec.break_force,spec.heat_conductance));
    };
    for z in 0..spec.nz { for y in 0..spec.ny {for x in 0..spec.nx {
        let a=index(x,y,z);
        if x+1<spec.nx {link(a,index(x+1,y,z),spec.pitch);}
        if y+1<spec.ny {link(a,index(x,y+1,z),spec.pitch);}
        if z+1<spec.nz {link(a,index(x,y,z+1),spec.pitch);}
        if spec.diagonals {
            let r=spec.pitch*2.0_f64.sqrt();
            if x+1<spec.nx && y+1<spec.ny {
                link(a,index(x+1,y+1,z),r);link(index(x+1,y,z),index(x,y+1,z),r);
            }
            if x+1<spec.nx && z+1<spec.nz {
                link(a,index(x+1,y,z+1),r);link(index(x+1,y,z),index(x,y,z+1),r);
            }
            if y+1<spec.ny && z+1<spec.nz {
                link(a,index(x,y+1,z+1),r);link(index(x,y+1,z),index(x,y,z+1),r);
            }
        }
    }}}
    Ok(ids)
}
#[cfg(test)]mod tests {
    use super::*;
    use crate::physics::STONE;
    #[test]fn correct_axial_topology(){
        let mut w=World::new();let mut s=LatticeSpec::experimental_block();s.nx=2;s.ny=2;s.nz=2;s.diagonals=false;
        let ids=add_lattice(&mut w,s,Vec3::new(0.,1.,0.),STONE,300.).unwrap();
        assert_eq!(ids.len(),8);assert_eq!(w.bonds.len(),12);
        assert!(w.bonds.iter().all(|b| b.rest==s.pitch));
    }
    #[test]fn no_hidden_recipes_or_added_mass(){
        let mut w=World::new();let s=LatticeSpec::experimental_block();
        add_lattice(&mut w,s,Vec3::new(0.,2.,0.),STONE,300.).unwrap();
        let mass=w.total_mass();w.step(.01);assert_eq!(mass,w.total_mass());
        assert!(w.bonds.iter().all(|b|b.a!=b.b));
    }
    #[test]fn rejects_invalid_geometry(){
        let mut w=World::new();let mut s=LatticeSpec::experimental_block();s.pitch=.1;
        assert!(add_lattice(&mut w,s,Vec3::ZERO,STONE,300.).is_err());
        assert_eq!(w.particles.len(),0);
    }
}
