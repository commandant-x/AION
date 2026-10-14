//! Sparse 3D subsurface edits on deterministic 2 metre cubic cells.
//! Material volume is tracked in integer voxel units (8m^3 each).
//! This is an exploratory voxel abstraction, not lithosphere continuum mechanics.
use std::{collections::BTreeMap,path::Path};
use crate::{World,earthworks::{Material,material_at}};

pub const VOXEL_SIZE_M:f64=2.0;
pub const VOXEL_VOLUME_M3:f64=8.0;
const MAGIC:&[u8;8]=b"AIONV014";
const MAX_OVERRIDES:usize=200_000;
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct VoxelKey {pub x:i32,pub y:i32,pub z:i32}
impl VoxelKey {
    pub fn at(x:f64,y:f64,z:f64)->Option<Self>{
        if ![x,y,z].iter().all(|v|v.is_finite() && v.abs()<1.0e8){return None;}
        Some(Self{x:(x/VOXEL_SIZE_M).floor() as i32,y:(y/VOXEL_SIZE_M).floor() as i32,z:(z/VOXEL_SIZE_M).floor() as i32})
    }
    pub fn center(self)->[f64;3]{[(self.x as f64+0.5)*VOXEL_SIZE_M,(self.y as f64+0.5)*VOXEL_SIZE_M,(self.z as f64+0.5)*VOXEL_SIZE_M]}
    pub fn neighbour(self,d:[i32;3])->Option<Self>{Some(Self{x:self.x.checked_add(d[0])?,y:self.y.checked_add(d[1])?,z:self.z.checked_add(d[2])?})}
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum CellEdit {Air,Placed(Material)}
#[derive(Debug,Clone)]
pub struct VoxelWorld {
    pub seed:u64,
    edits:BTreeMap<VoxelKey,CellEdit>,
    inventory:[u32;8],
    extracted:[u32;8],
    pub revision:u64,
}
#[derive(Debug,Clone,Copy)]
pub struct RayHit{pub solid:VoxelKey,pub last_air:Option<VoxelKey>}
impl VoxelWorld {
    pub fn new(seed:u64)->Self{Self{seed,edits:BTreeMap::new(),inventory:[0;8],extracted:[0;8],revision:0}}
    pub fn edited_count(&self)->usize{self.edits.len()}
    pub fn inventory_units(&self,m:Material)->u32{self.inventory[m as usize]}
    pub fn inventory_mass_kg(&self)->f64{Material::ALL.iter().map(|&m|self.inventory_units(m) as f64*VOXEL_VOLUME_M3*f64::from(m.density_kg_m3())).sum()}
    pub fn edited_in_chunk(&self,cx:i32,cz:i32)->impl Iterator<Item=(VoxelKey,CellEdit)>+'_ {
        self.edits.iter().filter_map(move |(&k,&v)|if k.x.div_euclid(128)==cx && k.z.div_euclid(128)==cz {Some((k,v))}else{None})
    }
    pub fn baseline(&self,world:&World,key:VoxelKey)->Option<Material>{
        let [x,y,z]=key.center();
        let original=world.elevation_m(x,z);
        if y>original{return None;}
        Some(material_at(world,x,z,(original-y).max(0.0)))
    }
    pub fn material(&self,world:&World,key:VoxelKey)->Option<Material>{
        match self.edits.get(&key){Some(CellEdit::Air)=>None,Some(CellEdit::Placed(m))=>Some(*m),None=>self.baseline(world,key)}
    }
    pub fn surface_open(&self,world:&World,x:f64,z:f64)->bool{
        let h=world.elevation_m(x,z);
        if let Some(k)=VoxelKey::at(x,((h-1.0)/VOXEL_SIZE_M).floor()*VOXEL_SIZE_M+0.5*VOXEL_SIZE_M,z){
            return self.edits.get(&k)==Some(&CellEdit::Air);
        }
        false
    }
    pub fn dig(&mut self,world:&World,key:VoxelKey)->Result<Material,String>{
        if world.seed!=self.seed {return Err("Graine incompatible".into());}
        let mat=self.material(world,key).ok_or("La cellule est déjà vide")?;
        if self.edits.len()>=MAX_OVERRIDES && !self.edits.contains_key(&key){return Err("Limite de cellules atteinte".into());}
        let mi=mat as usize;
        let new_inventory=self.inventory[mi].checked_add(1).ok_or("Inventaire plein")?;
        let prior=self.edits.get(&key).copied();
        let new_extracted=if prior.is_none(){Some(self.extracted[mi].checked_add(1).ok_or("Compteur d'extraction plein")?)}else{None};
        match prior {
            Some(CellEdit::Placed(_)) if self.baseline(world,key).is_none()=>{self.edits.remove(&key);},
            _=>{self.edits.insert(key,CellEdit::Air);},
        }
        self.inventory[mi]=new_inventory;
        if let Some(n)=new_extracted{self.extracted[mi]=n;}
        self.revision=self.revision.wrapping_add(1);
        debug_assert!(self.material_balance_ok());
        Ok(mat)
    }
    pub fn place(&mut self,world:&World,key:VoxelKey,m:Material)->Result<(),String>{
        if world.seed!=self.seed{return Err("Graine incompatible".into());}
        if self.material(world,key).is_some(){return Err("Cellule occupée".into());}
        if self.inventory[m as usize]==0{return Err(format!("Aucun {} disponible",m.name()));}
        if self.edits.len()>=MAX_OVERRIDES && !self.edits.contains_key(&key){return Err("Limite de cellules atteinte".into());}
        self.edits.insert(key,CellEdit::Placed(m));
        self.inventory[m as usize]-=1;
        self.revision=self.revision.wrapping_add(1);
        debug_assert!(self.material_balance_ok());
        Ok(())
    }
    pub fn raycast(&self,world:&World,start:[f64;3],dir:[f64;3],max_distance:f64)->Option<RayHit>{
        if !start.iter().chain(dir.iter()).all(|v|v.is_finite())||!max_distance.is_finite()||max_distance<=0.0{return None;}
        let len=dir.iter().map(|v|v*v).sum::<f64>().sqrt();if len<1e-10{return None;}
        let d=dir.map(|v|v/len);
        let mut previous=None;
        for i in 1..=((max_distance.min(60.0)/0.25) as usize){
            let t=i as f64*0.25;
            let k=VoxelKey::at(start[0]+d[0]*t,start[1]+d[1]*t,start[2]+d[2]*t)?;
            if self.material(world,k).is_some(){return Some(RayHit{solid:k,last_air:previous});}
            previous=Some(k);
        }
        None
    }
    pub fn material_balance_ok(&self)->bool{
        let mut placed=[0u64;8];
        for e in self.edits.values(){if let CellEdit::Placed(m)=e{placed[*m as usize]+=1;}}
        (0..8).all(|i|u64::from(self.extracted[i])==u64::from(self.inventory[i])+placed[i])
    }
    pub fn save(&self,path:&Path)->Result<(),String>{
        if !self.material_balance_ok()||self.edits.len()>MAX_OVERRIDES{return Err("Bilan de matière invalide".into());}
        let mut b=Vec::with_capacity(92+self.edits.len()*13);
        b.extend_from_slice(MAGIC);b.extend_from_slice(&self.seed.to_le_bytes());
        b.extend_from_slice(&(self.edits.len() as u32).to_le_bytes());
        for a in [&self.inventory,&self.extracted] {for &n in a {b.extend_from_slice(&n.to_le_bytes());}}
        for (&k,&v) in &self.edits {
            b.extend_from_slice(&k.x.to_le_bytes());b.extend_from_slice(&k.y.to_le_bytes());b.extend_from_slice(&k.z.to_le_bytes());
            b.push(match v{CellEdit::Air=>255,CellEdit::Placed(m)=>m as u8});
        }
        let sum=hash64(&b);b.extend_from_slice(&sum.to_le_bytes());
        std::fs::write(path,b).map_err(|e|e.to_string())
    }
    pub fn load(world:&World,path:&Path)->Result<Self,String>{
        let b=std::fs::read(path).map_err(|e|e.to_string())?;
        if b.len()<92||&b[..8]!=MAGIC{return Err("Format AIONV014 incorrect".into());}
        let u32_at=|i:usize|u32::from_le_bytes(b[i..i+4].try_into().unwrap());
        let seed=u64::from_le_bytes(b[8..16].try_into().unwrap());
        if seed!=world.seed{return Err("Graine différente".into());}
        let count=u32_at(16) as usize;
        if count>MAX_OVERRIDES||b.len()!=92+13*count{return Err("Longueur incorrecte".into());}
        let hash=u64::from_le_bytes(b[b.len()-8..].try_into().unwrap());
        if hash64(&b[..b.len()-8])!=hash{return Err("Sauvegarde endommagée".into());}
        let mut s=Self::new(seed);
        for i in 0..8{s.inventory[i]=u32_at(20+i*4);s.extracted[i]=u32_at(52+i*4);}
        for i in 0..count {
            let off=84+i*13;
            let key=VoxelKey{x:i32::from_le_bytes(b[off..off+4].try_into().unwrap()),y:i32::from_le_bytes(b[off+4..off+8].try_into().unwrap()),z:i32::from_le_bytes(b[off+8..off+12].try_into().unwrap())};
            let kind=b[off+12];
            let val=if kind==255{CellEdit::Air}else{CellEdit::Placed(Material::from_index(kind).ok_or("Matériau invalide")?)};
            if matches!(val,CellEdit::Air)&&s.baseline(world,key).is_none(){return Err("Vide superflu dans la sauvegarde".into());}
            if s.edits.insert(key,val).is_some(){return Err("Cellule dupliquée".into());}
        }
        let mut expected=[0u64;8];
        for (&key,e) in &s.edits {
            if let Some(m)=s.baseline(world,key){
                if matches!(e,CellEdit::Air|CellEdit::Placed(_)){expected[m as usize]+=1;}
            }
        }
        if (0..8).any(|i|expected[i]!=u64::from(s.extracted[i]))||!s.material_balance_ok(){return Err("Extraction incohérente".into());}
        s.revision=1;
        Ok(s)
    }
}
fn hash64(b:&[u8])->u64{b.iter().fold(0xcbf29ce484222325u64,|h,&v|(h^u64::from(v)).wrapping_mul(0x100000001b3))}

#[cfg(test)] mod tests {
use super::*;
fn top(world:&World,x:f64,z:f64)->VoxelKey { VoxelKey::at(x,((world.elevation_m(x,z)-1.0)/2.0).floor()*2.0+1.0,z).unwrap() }
#[test] fn cave_below_roof_and_vertical_tunnel(){
 let w=World::new(42);let mut v=VoxelWorld::new(w.seed);
 let k=top(&w,21.,19.);
 let under=k.neighbour([0,-1,0]).unwrap();
 assert!(v.material(&w,k).is_some());
 assert!(v.material(&w,under).is_some());
 let m=v.dig(&w,under).unwrap();
 assert_eq!(v.material(&w,under),None);
 assert!(v.material(&w,k).is_some(),"roof remains intact");
 v.dig(&w,k).unwrap();assert!(v.surface_open(&w,21.,19.));
 assert!(v.material_balance_ok());assert_eq!(v.inventory_units(m)>=1,true);
}
#[test] fn negative_coordinates_and_real_volume(){
 let w=World::new(17);let mut v=VoxelWorld::new(17);let k=top(&w,-21.,-45.);
 let m=v.dig(&w,k).unwrap();assert_eq!(v.inventory_mass_kg(),8.0*f64::from(m.density_kg_m3()));
 assert!(v.place(&w,k,m).is_ok());assert!(v.material_balance_ok());
 assert_eq!(v.inventory_units(m),0);
 assert_eq!(v.dig(&w,k).unwrap(),m);assert!(v.material_balance_ok());
}
#[test] fn tunnel_lateral_cells_are_truly_3d(){
 let w=World::new(18);let mut v=VoxelWorld::new(18);let k=top(&w,0.,0.).neighbour([0,-2,0]).unwrap();
 let n=k.neighbour([1,0,0]).unwrap();
 v.dig(&w,k).unwrap();v.dig(&w,n).unwrap();
 assert!(v.material(&w,k).is_none()&&v.material(&w,n).is_none());
 assert!(v.material(&w,k.neighbour([0,1,0]).unwrap()).is_some());
}
#[test] fn save_load_corruption_and_wrong_seed(){
 let w=World::new(102);let mut v=VoxelWorld::new(102);let k=top(&w,-10.,20.);
 let m=v.dig(&w,k).unwrap();let p=k.neighbour([2,2,0]).unwrap();assert!(v.material(&w,p).is_none());
 v.place(&w,p,m).unwrap();let path=std::env::temp_dir().join(format!("aionv014-{}-{}.bin",std::process::id(),w.seed));
 v.save(&path).unwrap();let loaded=VoxelWorld::load(&w,&path).unwrap();
 assert_eq!(loaded.edits,v.edits);assert!(loaded.material_balance_ok());
 assert!(VoxelWorld::load(&World::new(103),&path).is_err());
 let mut bad=std::fs::read(&path).unwrap();bad[30]^=0x40;std::fs::write(&path,&bad).unwrap();
 assert!(VoxelWorld::load(&w,&path).is_err());std::fs::remove_file(path).unwrap();
}
#[test] fn ray_hits_surface_and_detects_opening(){
 let w=World::new(18);let mut v=VoxelWorld::new(18);let h=w.elevation_m(3.,3.);
 let ray=v.raycast(&w,[3.,h+10.,3.],[0.,-1.,0.],20.).unwrap();
 assert_eq!(ray.solid,top(&w,3.,3.));
 v.dig(&w,ray.solid).unwrap();
 let next=v.raycast(&w,[3.,h+10.,3.],[0.,-1.,0.],25.).unwrap();
 assert_eq!(next.solid.y,ray.solid.y-1);
}
}