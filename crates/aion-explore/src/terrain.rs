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
    Mesh { vertices, indices }
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
        assert_eq!(m.vertices.len(),(CHUNK_QUADS+1).pow(2));
        assert_eq!(m.indices.len(),6*CHUNK_QUADS.pow(2));
        assert!(m.indices.iter().all(|&i| i < m.vertices.len() as u32));
        for tri in m.indices.chunks_exact(3) {
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
    #[test] fn streaming_prioritises_nearest_and_unique_chunks() {
        let v=ordered_neighbours(-2,7,3);
        assert_eq!(v.len(),49);
        assert_eq!(v[0],(-2,7));
        let set=std::collections::HashSet::<_>::from_iter(v.iter().copied());
        assert_eq!(set.len(),v.len());
    }
}