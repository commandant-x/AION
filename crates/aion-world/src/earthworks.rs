//! Experimental conserved earthworks. The landscape is a height field, not a 3D voxel geology.
//! Each global 2m vertex owns a compact 4m² footprint. Edits are quantized to 0.5m.
use std::collections::BTreeMap;
use std::path::Path;
use crate::World;

pub const EDIT_SPACING_M:f64=2.0;
pub const EDIT_STEP_M:f64=0.5;
const MAX_STEPS:u16=80;
const MAGIC:&[u8;8]=b"AIONED13";

#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
#[repr(u8)]
pub enum Material { Soil=0, Sand=1, Clay=2, Gravel=3, Peat=4, Limestone=5, Granite=6, Basalt=7 }
impl Material {
    pub const ALL:[Self;8]=[Self::Soil,Self::Sand,Self::Clay,Self::Gravel,Self::Peat,Self::Limestone,Self::Granite,Self::Basalt];
    pub fn from_index(n:u8)->Option<Self>{Self::ALL.get(n as usize).copied()}
    pub fn name(self)->&'static str { match self {Self::Soil=>"Terre",Self::Sand=>"Sable",Self::Clay=>"Argile",Self::Gravel=>"Gravier",Self::Peat=>"Tourbe",Self::Limestone=>"Calcaire",Self::Granite=>"Granite",Self::Basalt=>"Basalte"} }
    pub fn density_kg_m3(self)->u16{match self{Self::Soil=>1350,Self::Sand=>1600,Self::Clay=>1750,Self::Gravel=>1800,Self::Peat=>550,Self::Limestone=>2550,Self::Granite=>2700,Self::Basalt=>2900}}
    pub fn hardness(self)->u8{match self{Self::Peat=>1,Self::Soil=>2,Self::Sand=>2,Self::Clay=>3,Self::Gravel=>4,Self::Limestone=>6,Self::Granite=>8,Self::Basalt=>9}}
    pub fn color(self)->[f32;3]{match self{Self::Soil=>[0.27,0.20,0.11],Self::Sand=>[0.71,0.61,0.40],Self::Clay=>[0.49,0.31,0.23],Self::Gravel=>[0.39,0.39,0.35],Self::Peat=>[0.16,0.13,0.12],Self::Limestone=>[0.65,0.62,0.54],Self::Granite=>[0.43,0.46,0.48],Self::Basalt=>[0.20,0.22,0.26]}}
}
/// A synthetic soil profile linked to geology, slope, moisture and coast.
/// Not an erosion model or a chemistry solver.
pub fn material_at(world:&World,x:f64,z:f64,below_original_surface_m:f64)->Material {
    let s=world.sample(x,z);
    let g=world.geology(x,z);
    let top=if s.elevation_m < 5.0 {Material::Sand}
        else if s.moisture>0.68 && s.slope<0.12{Material::Peat}
        else if s.slope>0.72{Material::Gravel}
        else {Material::Soil};
    let soil_thickness=(2.2-s.slope*1.8).clamp(0.25,2.5);
    let subsoil_thickness=(3.8-s.slope*1.3).clamp(1.0,4.0);
    if below_original_surface_m<soil_thickness{return top;}
    if below_original_surface_m<soil_thickness+subsoil_thickness {
        return if s.moisture>0.46{Material::Clay}else{Material::Gravel};
    }
    if g.rock_hardness>0.69{return Material::Granite;}
    if g.compression>0.68 && g.rock_hardness>0.46{return Material::Basalt;}
    Material::Limestone
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
struct Cell { removed:u16, filled:u16, cap:Option<Material> }
impl Cell {fn empty()->Self {Self{removed:0,filled:0,cap:None}} fn offset(&self)->f64{(f64::from(self.filled)-f64::from(self.removed))*EDIT_STEP_M} }
#[derive(Debug,Clone)]
pub struct Earthworks {
    pub seed:u64,
    edits:BTreeMap<(i32,i32),Cell>,
    /// Each unit represents 2m³. Conservation checks are exact in integers.
    inventory:[u32;8],
    extracted:[u32;8],
    pub revision:u64,
}
impl Earthworks {
    pub fn new(seed:u64)->Self{Self{seed,edits:BTreeMap::new(),inventory:[0;8],extracted:[0;8],revision:0}}
    pub fn cell_count(&self)->usize{self.edits.len()}
    pub fn inventory_units(&self,m:Material)->u32{self.inventory[m as usize]}
    pub fn inventory_volume_m3(&self,m:Material)->f64{self.inventory_units(m) as f64*EDIT_SPACING_M.powi(2)*EDIT_STEP_M}
    pub fn inventory_mass_kg(&self)->f64{Material::ALL.iter().map(|&m|self.inventory_volume_m3(m)*f64::from(m.density_kg_m3())).sum()}
    pub fn offset_m(&self,x:f64,z:f64)->f64{
        let fx=x/EDIT_SPACING_M;let fz=z/EDIT_SPACING_M;
        let ix=fx.floor() as i32;let iz=fz.floor() as i32;
        let u=fx-ix as f64;let v=fz-iz as f64;
        let o=|x:i32,z:i32|self.edits.get(&(x,z)).map(Cell::offset).unwrap_or(0.0);
        o(ix,iz)*(1.0-u)*(1.0-v)+o(ix+1,iz)*u*(1.0-v)+o(ix,iz+1)*(1.0-u)*v+o(ix+1,iz+1)*u*v
    }
    pub fn elevation_m(&self,world:&World,x:f64,z:f64)->f64{world.elevation_m(x,z)+self.offset_m(x,z)}
    pub fn surface_material(&self,world:&World,x:f64,z:f64)->Material{
        let (ix,iz)=Self::key(x,z);
        let cell=self.edits.get(&(ix,iz)).copied().unwrap_or(Cell::empty());
        if cell.filled>0 {return cell.cap.expect("filled cell has a material");}
        material_at(world,ix as f64*EDIT_SPACING_M,iz as f64*EDIT_SPACING_M,f64::from(cell.removed)*EDIT_STEP_M)
    }
    fn key(x:f64,z:f64)->(i32,i32){((x/EDIT_SPACING_M).round() as i32,(z/EDIT_SPACING_M).round() as i32)}
    fn commit_cell(&mut self,key:(i32,i32),cell:Cell){
        if cell.removed==0 && cell.filled==0 {self.edits.remove(&key);} else {self.edits.insert(key,cell);}
        self.revision=self.revision.wrapping_add(1);
    }
    pub fn excavate(&mut self,world:&World,x:f64,z:f64)->Result<Material,String>{
        if self.seed!=world.seed||!x.is_finite()||!z.is_finite(){return Err("Monde incompatible".into());}
        let key=Self::key(x,z);let mut cell=self.edits.get(&key).copied().unwrap_or(Cell::empty());
        let material=if cell.filled>0 {
            let m=cell.cap.ok_or("Remblai sans matériau")?;
            cell.filled-=1;if cell.filled==0{cell.cap=None;}m
        }else{
            if cell.removed>=MAX_STEPS {return Err("Profondeur maximale 40 m".into());}
            let depth=(f64::from(cell.removed)+0.5)*EDIT_STEP_M;
            let m=material_at(world,key.0 as f64*EDIT_SPACING_M,key.1 as f64*EDIT_SPACING_M,depth);
            cell.removed+=1;self.extracted[m as usize]=self.extracted[m as usize].checked_add(1).ok_or("Stock saturé")?;m
        };
        self.inventory[material as usize]=self.inventory[material as usize].checked_add(1).ok_or("Stock saturé")?;
        self.commit_cell(key,cell);Ok(material)
    }
    pub fn deposit(&mut self,world:&World,x:f64,z:f64,m:Material)->Result<(),String>{
        if self.seed!=world.seed||!x.is_finite()||!z.is_finite(){return Err("Monde incompatible".into());}
        if self.inventory[m as usize]==0{return Err(format!("Aucun {} dans l'inventaire",m.name()));}
        let key=Self::key(x,z);let mut cell=self.edits.get(&key).copied().unwrap_or(Cell::empty());
        if cell.filled>=MAX_STEPS{return Err("Hauteur de remblai maximale".into());}
        if cell.filled>0 && cell.cap!=Some(m){return Err("Retirer d'abord le matériau déjà déposé".into());}
        cell.filled+=1;cell.cap=Some(m);
        self.inventory[m as usize]-=1;self.commit_cell(key,cell);Ok(())
    }
    pub fn material_balance_ok(&self)->bool{
        let mut placed=[0u64;8];
        for cell in self.edits.values(){if cell.filled>0{let Some(m)=cell.cap else{return false};placed[m as usize]+=u64::from(cell.filled);}else if cell.cap.is_some(){return false}}
        (0..8).all(|i|u64::from(self.extracted[i])==u64::from(self.inventory[i])+placed[i])
    }
    pub fn save(&self,path:&Path)->Result<(),String>{
        if !self.material_balance_ok(){return Err("Conservation de matière invalide".into());}
        let mut b=Vec::with_capacity(100+self.edits.len()*13);
        b.extend_from_slice(MAGIC);b.extend_from_slice(&self.seed.to_le_bytes());
        b.extend_from_slice(&(self.edits.len() as u32).to_le_bytes());
        for a in [&self.inventory,&self.extracted] {for &v in a{b.extend_from_slice(&v.to_le_bytes());}}
        for (&(x,z),c) in &self.edits {
            b.extend_from_slice(&x.to_le_bytes());b.extend_from_slice(&z.to_le_bytes());
            b.extend_from_slice(&c.removed.to_le_bytes());b.extend_from_slice(&c.filled.to_le_bytes());
            b.push(c.cap.map(|m|m as u8).unwrap_or(255));
        }
        let hash=hash64(&b);b.extend_from_slice(&hash.to_le_bytes());
        std::fs::write(path,b).map_err(|e|e.to_string())
    }
    pub fn load(world:&World,path:&Path)->Result<Self,String>{
        let b=std::fs::read(path).map_err(|e|e.to_string())?;
        if b.len()<92||&b[0..8]!=MAGIC{return Err("Fichier AIONED13 invalide".into());}
        let u32_at=|i:usize|u32::from_le_bytes(b[i..i+4].try_into().unwrap());
        let seed=u64::from_le_bytes(b[8..16].try_into().unwrap());
        if seed!=world.seed{return Err("Graine du monde différente".into());}
        let count=u32_at(16) as usize;
        if count>1_000_000 || b.len()!=92+count*13{return Err("Taille de sauvegarde invalide".into());}
        let checksum=u64::from_le_bytes(b[b.len()-8..].try_into().unwrap());
        if hash64(&b[..b.len()-8])!=checksum{return Err("Sauvegarde altérée".into());}
        let mut e=Self::new(seed);
        for i in 0..8{e.inventory[i]=u32_at(20+i*4);e.extracted[i]=u32_at(52+i*4);}
        for i in 0..count {
            let j=84+i*13;
            let x=i32::from_le_bytes(b[j..j+4].try_into().unwrap());
            let z=i32::from_le_bytes(b[j+4..j+8].try_into().unwrap());
            let removed=u16::from_le_bytes(b[j+8..j+10].try_into().unwrap());
            let filled=u16::from_le_bytes(b[j+10..j+12].try_into().unwrap());
            let cap=if b[j+12]==255{None}else{Material::from_index(b[j+12])};
            if removed>MAX_STEPS||filled>MAX_STEPS||(filled>0)!=cap.is_some()||(removed==0&&filled==0) {
                return Err("Cellule de terrain invalide".into());
            }
            if e.edits.insert((x,z),Cell{removed,filled,cap}).is_some(){return Err("Cellules en double".into());}
        }
        if !e.material_balance_ok(){return Err("Masse non conservée".into());}
        // Verify extracted layers against the deterministic world (no material fabricated).
        let mut from_earth=[0u64;8];
        for (&(x,z),cell) in &e.edits{for n in 0..cell.removed {
            let m=material_at(world,x as f64*EDIT_SPACING_M,z as f64*EDIT_SPACING_M,(f64::from(n)+0.5)*EDIT_STEP_M);
            from_earth[m as usize]+=1;
        }}
        if (0..8).any(|i|from_earth[i]!=u64::from(e.extracted[i])){return Err("Couches extraites incohérentes".into());}
        e.revision=1;
        Ok(e)
    }
}
fn hash64(bytes:&[u8])->u64{bytes.iter().fold(0xcbf29ce484222325u64,|h,&x|(h^u64::from(x)).wrapping_mul(0x100000001b3))}
#[cfg(test)] mod tests {
use super::*;
#[test]fn deterministic_profiles_and_different_ground(){
 let a=World::new(31);let b=World::new(31);
 for &(x,z) in &[(-500.,70.),(16.,-93.),(2052.,411.)] { for d in [0.,1.,4.,12.,39.] {
 assert_eq!(material_at(&a,x,z,d),material_at(&b,x,z,d));
 }}
 let mut kinds=std::collections::BTreeSet::new();for x in -20..20{for z in -20..20{for depth in [0.0,2.0,8.0,20.0] {kinds.insert(material_at(&a,x as f64*443.,z as f64*553.,depth));}}}
 assert!(kinds.len()>=3);assert!(Material::ALL.iter().all(|m|m.density_kg_m3()>0&&m.hardness()>0));
}
#[test]fn conserved_excavation_and_deposit(){
 let w=World::new(72);let mut e=Earthworks::new(w.seed);
 let m=e.excavate(&w,-2.,4.).unwrap();let mass=m.density_kg_m3() as f64*2.0;
 assert_eq!(e.inventory_mass_kg(),mass);assert_eq!(e.offset_m(-2.,4.),-0.5);
 assert!(e.deposit(&w,14.,-6.,m).is_ok());assert_eq!(e.inventory_mass_kg(),0.0);
 assert_eq!(e.offset_m(14.,-6.),0.5);assert!(e.material_balance_ok());
 assert!(e.deposit(&w,13.,-6.,m).is_err());
 let n=e.excavate(&w,14.,-6.).unwrap();assert_eq!(m,n);assert!(e.material_balance_ok());
}
#[test]fn different_depths_and_no_underflow(){
 let w=World::new(19);let mut e=Earthworks::new(w.seed);
 for _ in 0..80 {e.excavate(&w,0.,0.).unwrap();}
 assert!(e.excavate(&w,0.,0.).is_err());assert!(e.material_balance_ok());
 assert_eq!(e.offset_m(0.,0.),-40.0);
}
#[test]fn negative_coords_and_chunk_boundary_are_continuous(){
 let w=World::new(20);let mut e=Earthworks::new(w.seed);
 e.excavate(&w,-256.,-2.).unwrap();
 assert!((e.offset_m(-257.,-2.)-e.offset_m(-255.,-2.)).abs()<1e-8);
 assert!((e.elevation_m(&w,-256.,-2.)-w.elevation_m(-256.,-2.)+0.5).abs()<1e-6);
}
#[test]fn save_load_and_bitflip_rejected(){
 let w=World::new(4);let mut e=Earthworks::new(w.seed);
 for _ in 0..4{e.excavate(&w,-17.,3.).unwrap();}let m=e.excavate(&w,3.,7.).unwrap();e.deposit(&w,-9.,-1.,m).unwrap();
 let path=std::env::temp_dir().join(format!("aioned13-{}-{}.bin",std::process::id(),w.seed));
 e.save(&path).unwrap();let restored=Earthworks::load(&w,&path).unwrap();
 assert_eq!(restored.edits,e.edits);assert_eq!(restored.inventory,e.inventory);
 assert!(restored.material_balance_ok());
 let mut bad=std::fs::read(&path).unwrap();bad[35]^=0x5a;std::fs::write(&path,&bad).unwrap();
 assert!(Earthworks::load(&w,&path).is_err());std::fs::remove_file(path).unwrap();
}
}