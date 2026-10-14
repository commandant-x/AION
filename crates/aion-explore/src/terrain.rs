//! Deterministic indexed surface mesh for the multiresolution world model.
//! Geometry (including chunk coordinates) is defined in metres.
use aion_world::{Biome, Chunk, CELL_METRES, CHUNK_QUADS, CHUNK_METRES};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub const ATTRS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x3
    ];
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

fn shade(b: Biome, altitude: f32, steep: f32) -> [f32; 3] {
    let (r,g,bb) = match b {
        Biome::Ocean   => (0.09,0.20,0.32),
        Biome::Shore   => (0.76,0.70,0.51),
        Biome::Wetland => (0.20,0.29,0.18),
        Biome::Desert  => (0.71,0.59,0.36),
        Biome::Grassland => (0.30,0.44,0.20),
        Biome::Forest => (0.16,0.33,0.17),
        Biome::Boreal => (0.23,0.34,0.27),
        Biome::Tundra => (0.53,0.56,0.49),
        Biome::Alpine => (0.48,0.48,0.46),
    };
    // Steep faces are rocky; high altitudes become snow-covered.
    let rock = ((steep - 0.48)*1.20).clamp(0.0,0.74);
    let snow = ((altitude-850.0)/450.0).clamp(0.0,0.88);
    let mix = |x:f32,y:f32,t:f32| x*(1.0-t)+y*t;
    [mix(mix(r,0.42,rock),0.88,snow),
     mix(mix(g,0.41,rock),0.91,snow),
     mix(mix(bb,0.40,rock),0.90,snow)]
}

pub fn build_mesh(c: &Chunk) -> Mesh {
    let mut vertices=Vec::with_capacity(Chunk::SIDE*Chunk::SIDE);
    for z in 0..Chunk::SIDE { for x in 0..Chunk::SIDE {
        let i=Chunk::index(x,z);
        let n=c.normals[i];
        let h=c.heights[i];
        vertices.push(Vertex {
            position: [(c.x as f64*CHUNK_METRES + x as f64*CELL_METRES) as f32, h, (c.z as f64*CHUNK_METRES + z as f64*CELL_METRES) as f32],
            normal: n,
            color: shade(c.biomes[i],h,(1.0/n[1].max(0.01)-1.0).max(0.0)),
        });
    }}
    let mut indices=Vec::with_capacity(CHUNK_QUADS*CHUNK_QUADS*6);
    for z in 0..CHUNK_QUADS { for x in 0..CHUNK_QUADS {
        let a=Chunk::index(x,z) as u32;
        let b=Chunk::index(x+1,z) as u32;
        let c=Chunk::index(x,z+1) as u32;
        let d=Chunk::index(x+1,z+1) as u32;
        // Surface winding for +Y facing triangles.
        indices.extend_from_slice(&[a,c,b,b,c,d]);
    }}
    // These trees are visual landmarks only. Biology/ecology remain future systems.
    append_procedural_trees(c,&mut vertices,&mut indices);
    Mesh { vertices, indices }
}

fn tree_hash(wx: i64, wz: i64) -> u64 {
    let mut h=(wx as u64).wrapping_mul(0x9e3779b97f4a7c15) ^
        (wz as u64).wrapping_mul(0xd1b54a32d192ed03);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^ (h >> 31)
}
fn add_face(v:&mut Vec<Vertex>,i:&mut Vec<u32>,a:[f32;3],b:[f32;3],c:[f32;3],color:[f32;3]) {
    let ab=[b[0]-a[0],b[1]-a[1],b[2]-a[2]];
    let ac=[c[0]-a[0],c[1]-a[1],c[2]-a[2]];
    let n=[ab[1]*ac[2]-ab[2]*ac[1],ab[2]*ac[0]-ab[0]*ac[2],ab[0]*ac[1]-ab[1]*ac[0]];
    let len=(n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt().max(1.0e-5);
    let normal=[n[0]/len,n[1]/len,n[2]/len];
    let base=v.len() as u32;
    for position in [a,b,c]{v.push(Vertex{position,normal,color});}
    i.extend_from_slice(&[base,base+1,base+2]);
}
fn append_procedural_trees(c:&Chunk,v:&mut Vec<Vertex>,i:&mut Vec<u32>) {
    for iz in (2..CHUNK_QUADS).step_by(3) {for ix in (2..CHUNK_QUADS).step_by(3) {
        let id=Chunk::index(ix,iz);
        let biome=c.biomes[id];
        let gx=c.x as i64*CHUNK_QUADS as i64+ix as i64;
        let gz=c.z as i64*CHUNK_QUADS as i64+iz as i64;
        let h=tree_hash(gx,gz);
        let density=match biome {
            Biome::Forest=>68,
            Biome::Boreal=>52,
            Biome::Grassland=>5,
            Biome::Wetland=>17,
            _=>0,
        };
        if h % 100 >= density {continue;}
        let x=(c.x as f64*CHUNK_METRES + ix as f64*CELL_METRES) as f32;
        let z=(c.z as f64*CHUNK_METRES + iz as f64*CELL_METRES) as f32;
        let base=c.heights[id];
        let height=9.0+((h>>8)%9) as f32;
        let crown=height*0.85;
        let radius=1.7+((h>>16)%100) as f32*0.017;
        let bark=[0.32,0.24,0.16];
        let leaf=if biome==Biome::Boreal{[0.12,0.27,0.22]}else{[0.11,0.33,0.16]};
        // A tapered trunk and two conifer silhouettes; not collision geometry.
        let y=base;
        let a=[x-0.25,y,z];let b=[x+0.25,y,z];
        let c0=[x+0.25,y+height*0.58,z];let d=[x-0.25,y+height*0.58,z];
        add_face(v,i,a,b,c0,bark);add_face(v,i,a,c0,d,bark);
        for (cy,rr,hh) in [(y+height*0.27,radius,crown*0.68),(y+height*0.52,radius*0.75,crown*0.56)]{
            let apex=[x,cy+hh,z];
            let ring=[[x-rr,cy,z-rr],[x+rr,cy,z-rr],[x+rr,cy,z+rr],[x-rr,cy,z+rr]];
            for k in 0..4 {add_face(v,i,ring[k],ring[(k+1)%4],apex,leaf);}
        }
    }}
}


pub fn chunk_key(x:f64,z:f64)->(i32,i32) {
    ((x/CHUNK_METRES).floor() as i32,(z/CHUNK_METRES).floor() as i32)
}

pub fn ordered_neighbours(cx:i32,cz:i32,radius:i32)->Vec<(i32,i32)> {
    let mut keys=Vec::new();
    for dz in -radius..=radius { for dx in -radius..=radius {
        keys.push((cx+dx,cz+dz));
    }}
    keys.sort_by_key(|&(x,z)| {
        let dx=x as i64-cx as i64;
        let dz=z as i64-cz as i64;
        (dx*dx+dz*dz,z,x)
    });
    keys
}

#[cfg(test)] mod tests {
    use super::*;
    use aion_world::World;

    #[test] fn coordinate_math_crosses_negative_boundaries() {
        assert_eq!(chunk_key(0.,0.),(0,0));
        assert_eq!(chunk_key(-1.,-1.),(-1,-1));
        assert_eq!(chunk_key(CHUNK_METRES,0.),(1,0));
        assert_eq!(chunk_key(-CHUNK_METRES,0.),(-1,0));
        assert_eq!(chunk_key(-CHUNK_METRES-0.1,0.),(-2,0));
    }
    #[test] fn mesh_indices_in_range_and_facing_up() {
        let m=build_mesh(&World::new(1234).chunk(-1,0));
        assert!(m.vertices.len()>=(CHUNK_QUADS+1).pow(2));
        assert!(m.indices.len()>=6*CHUNK_QUADS.pow(2));
        assert!(m.indices.iter().all(|&i| i < m.vertices.len() as u32));
        for tri in m.indices[..6*CHUNK_QUADS.pow(2)].chunks_exact(3) {
            let a=m.vertices[tri[0] as usize].position;
            let b=m.vertices[tri[1] as usize].position;
            let c=m.vertices[tri[2] as usize].position;
            let ux=b[0]-a[0];let uz=b[2]-a[2];
            let vx=c[0]-a[0];let vz=c[2]-a[2];
            assert!(uz*vx-ux*vz > 0.);
        }
        assert!(m.vertices.iter().flat_map(|v|v.color).all(|x|x.is_finite() && (0.0..=1.0).contains(&x)));
    }
    #[test] fn chunk_edges_are_identical_in_global_coordinates() {
        let world=World::new(444);
        let a=build_mesh(&world.chunk(-1,0));let b=build_mesh(&world.chunk(0,0));
        for z in 0..=CHUNK_QUADS {
            let va=a.vertices[Chunk::index(CHUNK_QUADS,z)];
            let vb=b.vertices[Chunk::index(0,z)];
            assert_eq!(va.position[1].to_bits(),vb.position[1].to_bits());
            assert_eq!(va.normal,vb.normal);
        }
    }
    #[test] fn decorative_geometry_is_deterministic_and_finite() {
        let w=World::new(101);
        let a=build_mesh(&w.chunk(2,3));
        let b=build_mesh(&w.chunk(2,3));
        assert_eq!(a.indices,b.indices);
        assert_eq!(bytemuck::cast_slice::<Vertex,u8>(&a.vertices),bytemuck::cast_slice::<Vertex,u8>(&b.vertices));
        assert!(a.vertices.iter().flat_map(|v|v.position).all(|p|p.is_finite()));
    }
    #[test] fn streaming_prioritises_nearest_and_unique_chunks() {
        let v=ordered_neighbours(-2,7,3);
        assert_eq!(v.len(),49);
        assert_eq!(v[0],(-2,7));
        let set=std::collections::HashSet::<_>::from_iter(v.iter().copied());
        assert_eq!(set.len(),v.len());
    }
}