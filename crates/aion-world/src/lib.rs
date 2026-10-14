//! PRIMORDIA: deterministic terrain, synthetic geologic structure and coarse biomes.
//! These are geological process-informed proxies, NOT a plate-tectonics or hydrology solver.

pub mod earthworks;
pub mod voxels;
pub use voxels::{VoxelWorld,VoxelKey,CellEdit,RayHit,VOXEL_VOLUME_M3};
pub use earthworks::{Earthworks,Material,material_at};

pub const SEA_LEVEL_M: f32 = 0.0;
pub const CHUNK_QUADS: usize = 32;
pub const CELL_METRES: f64 = 8.0;
pub const CHUNK_METRES: f64 = CHUNK_QUADS as f64 * CELL_METRES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome { Ocean, Shore, Wetland, Desert, Grassland, Forest, Boreal, Tundra, Alpine }

/// Deterministic geological controls used to shape terrain. No evolution over time yet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geology {
    /// Broad fault-belt uplift proxy, varying over regional distances.
    pub compression: f64,
    /// Resistance to mechanical/chemical erosion (0..1 proxy).
    pub rock_hardness: f64,
    /// Incised valley depth following coherent meandering drainage corridors.
    pub valley_incision_m: f64,
    pub uplift_m: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub elevation_m: f64,
    pub temperature_c: f64,
    pub moisture: f64,
    pub slope: f64,
    pub biome: Biome,
}
#[derive(Debug, Clone)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub heights: Vec<f32>,
    pub biomes: Vec<Biome>,
    pub normals: Vec<[f32; 3]>,
}
impl Chunk {
    pub const SIDE: usize = CHUNK_QUADS + 1;
    pub fn index(x: usize, z: usize) -> usize { z * Self::SIDE + x }
    pub fn height(&self, x: usize, z: usize) -> f32 { self.heights[Self::index(x, z)] }
    pub fn to_obj(&self) -> String {
        let mut out = format!("# PRIMORDIA chunk {}, {}\n", self.x, self.z);
        for z in 0..Self::SIDE {
            for x in 0..Self::SIDE {
                let wx = self.x as f64 * CHUNK_METRES + x as f64 * CELL_METRES;
                let wz = self.z as f64 * CHUNK_METRES + z as f64 * CELL_METRES;
                out.push_str(&format!("v {:.3} {:.3} {:.3}\n",wx,self.height(x,z),wz));
            }
        }
        for n in &self.normals { out.push_str(&format!("vn {:.6} {:.6} {:.6}\n",n[0],n[1],n[2])); }
        for z in 0..CHUNK_QUADS {
            for x in 0..CHUNK_QUADS {
                let a=Self::index(x,z)+1; let b=Self::index(x+1,z)+1;
                let c=Self::index(x,z+1)+1; let d=Self::index(x+1,z+1)+1;
                out.push_str(&format!("f {a}//{a} {c}//{c} {b}//{b}\nf {b}//{b} {c}//{c} {d}//{d}\n"));
            }
        }
        out
    }
}
#[derive(Clone, Copy)]
pub struct World { pub seed: u64 }
impl World {
    pub fn new(seed: u64) -> Self { Self { seed } }
    fn noise(&self, x: f64, z: f64, frequency: f64, salt: u64) -> f64 {
        let fx=x*frequency; let fz=z*frequency;
        let ix=fx.floor() as i64; let iz=fz.floor() as i64;
        let u=smooth(fx-ix as f64); let v=smooth(fz-iz as f64);
        let s=self.seed^salt;
        lerp(lerp(random(ix,iz,s),random(ix+1,iz,s),u),
             lerp(random(ix,iz+1,s),random(ix+1,iz+1,s),u),v)
    }
    fn fbm(&self, x:f64,z:f64, base:f64, salt:u64, octaves:usize)->f64 {
        let mut value=0.0; let mut weight=0.0;
        let mut amplitude=1.0; let mut frequency=base;
        for octave in 0..octaves {
            value+=amplitude*self.noise(x,z,frequency,salt.wrapping_add(octave as u64*65537));
            weight+=amplitude; amplitude*=0.48; frequency*=2.04;
        }
        value/weight
    }
    pub fn geology(&self,x:f64,z:f64)->Geology {
        // Curved, elongated deformation belts. These do NOT simulate the
        // velocities, material properties or chronology of real tectonic plates.
        let offset=self.fbm(x,z,0.00017,0x5A91,3)*900.0;
        let belt=self.fbm(x+offset,z-offset,0.00043,0x5A92,3).abs();
        let compression=((0.36-belt)/0.36).clamp(0.0,1.0).powi(2);
        let rock_hardness=((self.fbm(x,z,0.00095,0x5A93,3)+1.0)*0.5).clamp(0.0,1.0);
        // Drainage corridors meander spatially; incision attenuates in hard rock.
        let channel=self.fbm(x+offset,z-offset,0.00076,0x5A94,4)
            +0.20*self.fbm(x,z,0.0032,0x5A95,2);
        let channel_mask=((0.095-channel.abs())/0.095).clamp(0.0,1.0).powi(2);
        let valley_incision_m=channel_mask*(42.0-24.0*rock_hardness);
        Geology {compression,rock_hardness,valley_incision_m,uplift_m:compression*420.0}
    }
    pub fn elevation_m(&self,x:f64,z:f64)->f64 {
        assert!(x.is_finite() && z.is_finite());
        let continent=self.fbm(x,z,0.00013,0x4101,5);
        let regional=self.fbm(x,z,0.00085,0x4102,4);
        let ridge=1.0-self.fbm(x,z,0.0005,0x4103,4).abs();
        let micro=self.fbm(x,z,0.008,0x4104,3);
        let geo=self.geology(x,z);
        let mountains=((continent+0.25)*1.65).clamp(0.0,1.0);
        // Broad crustal basins, then deformed bedrock, then valley incision.
        let base=continent*1450.0+regional*145.0-95.0;
        let folds=(mountains*mountains*ridge*400.0) + geo.uplift_m*mountains;
        let weathering=micro*(10.0+20.0*geo.rock_hardness);
        let fluvial=geo.valley_incision_m * (1.0-0.4*geo.rock_hardness);
        base+folds+weathering-fluvial
    }
    pub fn biome_at(&self,x:f64,z:f64,h:f64,slope:f64)->Biome {
        let moisture=((self.fbm(x,z,0.00065,0xB100,5)+1.0)*0.5).clamp(0.0,1.0);
        let latitude=(z/120_000.0).clamp(-1.0,1.0).abs();
        let temp=29.0-latitude*54.0-h.max(0.0)*0.0065;
        if h < SEA_LEVEL_M as f64-4.0 { Biome::Ocean }
        else if h < SEA_LEVEL_M as f64+3.0 { Biome::Shore }
        else if slope > 1.15 || (temp < -12.0 && h > 500.0) { Biome::Alpine }
        else if temp < -8.0 { Biome::Tundra }
        else if temp < 5.0 { Biome::Boreal }
        else if moisture > 0.73 && slope < 0.13 { Biome::Wetland }
        else if moisture < 0.30 { Biome::Desert }
        else if moisture > 0.51 { Biome::Forest }
        else { Biome::Grassland }
    }
    pub fn sample(&self,x:f64,z:f64)->Sample {
        let h=self.elevation_m(x,z);
        let moisture=((self.fbm(x,z,0.00065,0xB100,5)+1.0)*0.5).clamp(0.0,1.0);
        let latitude=(z/120_000.0).clamp(-1.0,1.0).abs();
        let temp=29.0-latitude*54.0-h.max(0.0)*0.0065;
        let dx=(self.elevation_m(x+CELL_METRES,z)-self.elevation_m(x-CELL_METRES,z))/(2.0*CELL_METRES);
        let dz=(self.elevation_m(x,z+CELL_METRES)-self.elevation_m(x,z-CELL_METRES))/(2.0*CELL_METRES);
        let slope=(dx*dx+dz*dz).sqrt();
        let biome=self.biome_at(x,z,h,slope);
        Sample { elevation_m:h,temperature_c:temp,moisture,slope,biome }
    }
    pub fn chunk(&self,x:i32,z:i32)->Chunk {
        let mut heights=Vec::with_capacity(Chunk::SIDE*Chunk::SIDE);
        let mut biomes=Vec::with_capacity(Chunk::SIDE*Chunk::SIDE);
        let mut normals=Vec::with_capacity(Chunk::SIDE*Chunk::SIDE);
        for iz in 0..Chunk::SIDE { for ix in 0..Chunk::SIDE {
            let wx=x as f64*CHUNK_METRES+ix as f64*CELL_METRES;
            let wz=z as f64*CHUNK_METRES+iz as f64*CELL_METRES;
            let sample=self.sample(wx,wz);
            let dx=(self.elevation_m(wx+CELL_METRES,wz)-self.elevation_m(wx-CELL_METRES,wz))/(2.0*CELL_METRES);
            let dz=(self.elevation_m(wx,wz+CELL_METRES)-self.elevation_m(wx,wz-CELL_METRES))/(2.0*CELL_METRES);
            let inv=1.0/(1.0+dx*dx+dz*dz).sqrt();
            heights.push(sample.elevation_m as f32);
            biomes.push(sample.biome);
            normals.push([(-dx*inv) as f32,inv as f32,(-dz*inv) as f32]);
        }}
        Chunk {x,z,heights,biomes,normals}
    }
}
fn smooth(t:f64)->f64 { t*t*(3.0-2.0*t) }
fn lerp(a:f64,b:f64,t:f64)->f64 { a+(b-a)*t }
fn splitmix64(mut v:u64)->u64 {
    v=v.wrapping_add(0x9e3779b97f4a7c15);
    v=(v^(v>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    v=(v^(v>>27)).wrapping_mul(0x94d049bb133111eb);
    v^(v>>31)
}
fn random(x:i64,z:i64,s:u64)->f64 {
    let h=splitmix64((x as u64).wrapping_mul(0xd6e8feb86659fd93)^
                     (z as u64).wrapping_mul(0xa5a3564e27f886b3)^s);
    ((h>>11) as f64)*(2.0/((1u64<<53) as f64))-1.0
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn deterministic_seed() {
        let a=World::new(43);let b=World::new(43);
        for (x,z) in [(-1.0,-1.0),(0.0,0.0),(2560.0,-456.0)] {
            assert_eq!(a.sample(x,z),b.sample(x,z));
        }
        assert_ne!(a.sample(1234.0,5678.0),World::new(44).sample(1234.0,5678.0));
    }
    #[test] fn seamless_borders_even_across_negative_coordinates() {
        let w=World::new(777);
        for (x,z) in [(-11,-12),(0,0),(8,-3)] {
            let a=w.chunk(x,z);let east=w.chunk(x+1,z);let north=w.chunk(x,z+1);
            for i in 0..=CHUNK_QUADS {
                assert_eq!(a.height(CHUNK_QUADS,i).to_bits(),east.height(0,i).to_bits());
                assert_eq!(a.height(i,CHUNK_QUADS).to_bits(),north.height(i,0).to_bits());
                assert_eq!(a.biomes[Chunk::index(CHUNK_QUADS,i)],east.biomes[Chunk::index(0,i)]);
            }
        }
    }
    #[test] fn finite_normalized_geometry() {
        for seed in [0,1,u64::MAX] {
            let c=World::new(seed).chunk(-3,7);
            assert_eq!(c.heights.len(),Chunk::SIDE*Chunk::SIDE);
            for (h,n) in c.heights.iter().zip(&c.normals) {
                assert!(h.is_finite() && h.abs()<12_000.0);
                let m=n.iter().map(|v|v*v).sum::<f32>().sqrt();
                assert!((m-1.0).abs()<1e-5);
            }
        }
    }
    #[test] fn obj_has_expected_mesh() {
        let obj=World::new(21).chunk(0,0).to_obj();
        assert_eq!(obj.lines().filter(|l|l.starts_with("v ")).count(),Chunk::SIDE*Chunk::SIDE);
        assert_eq!(obj.lines().filter(|l|l.starts_with("f ")).count(),CHUNK_QUADS*CHUNK_QUADS*2);
    }
    #[test] fn bounded_climate_proxy() {
        let w=World::new(100);
        for i in -10..=10 {for j in -10..=10 {
            let s=w.sample(i as f64*520.0,j as f64*520.0);
            assert!((0.0..=1.0).contains(&s.moisture) && s.slope>=0.0 && s.temperature_c.is_finite());
        }}
    }
    #[test] fn geological_controls_are_bounded_repeatable_and_spatially_coherent() {
        let a=World::new(735);
        for &(x,z) in &[(-5400.0,2200.0),(0.0,0.0),(430.0,-880.0)] {
            let g=a.geology(x,z);
            assert_eq!(g,a.geology(x,z));
            assert!((0.0..=1.0).contains(&g.compression));
            assert!((0.0..=1.0).contains(&g.rock_hardness));
            assert!((0.0..=42.0).contains(&g.valley_incision_m));
            assert!(g.uplift_m>=0.0 && g.uplift_m<=420.0);
            assert!((a.elevation_m(x,z)-a.elevation_m(x+0.1,z)).abs()<10.0);
        }
    }
    #[test] fn terrain_has_ocean_basin_and_emerged_land() {
        let w=World::new(42);
        let mut sea=false;let mut land=false;
        for z in (-18..=18).step_by(3) {for x in (-18..=18).step_by(3) {
            let h=w.elevation_m(x as f64*256.0,z as f64*256.0);
            sea|=h<SEA_LEVEL_M as f64-5.0;
            land|=h>SEA_LEVEL_M as f64+5.0;
        }}
        assert!(sea && land, "coastline should exist within the test region");
    }

}