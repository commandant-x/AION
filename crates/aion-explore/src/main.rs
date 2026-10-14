//! PRIMORDIA: first native GPU terrain exploration. Not yet full simulation.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod camera;
mod terrain;

use std::{collections::HashMap, path::PathBuf, sync::{Arc,RwLock,mpsc::{self,Receiver,Sender}}, time::{Duration,Instant}};
use aion_world::{World,Earthworks,Material,VoxelWorld,VOXEL_VOLUME_M3,CHUNK_METRES};
use bytemuck::{Pod,Zeroable};
use glam::{Vec3,Mat4};
use wgpu::util::DeviceExt;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow,EventLoop},
    keyboard::{KeyCode,PhysicalKey},
    window::{CursorGrabMode,WindowBuilder,Window},
};
use camera::{Camera,Keys};
use terrain::{Mesh, Vertex,build_mesh_lod,build_mesh_lod_edited,build_mesh_with_voxels,water_mesh,lod_at,chunk_key,ordered_neighbours};

const RADIUS: i32=3; // up to 49 chunks / ~1.8 km square resident
const DEFAULT_SEED:u64=42;

#[repr(C)]
#[derive(Clone,Copy,Pod,Zeroable)]
struct Uniforms {
    vp: [[f32;4];4],
    eye: [f32;4],
    sun: [f32;4],
}

type TerrainJob=(i32,i32,u8,u64);
type TerrainResult=(i32,i32,u8,u64,Mesh);
fn spawn_chunk_worker(seed:u64,earth:Arc<RwLock<Earthworks>>,voxels:Arc<RwLock<VoxelWorld>>)->Result<(Sender<TerrainJob>,Receiver<TerrainResult>),String> {
    let (job_sender,job_receiver)=mpsc::channel::<TerrainJob>();
    let (result_sender,result_receiver)=mpsc::channel::<TerrainResult>();
    std::thread::Builder::new().name("aion-terrain-builder".into()).spawn(move || {
        let world=World::new(seed);
        while let Ok((x,z,lod,requested_revision))=job_receiver.recv() {
            let snapshot=earth.read().expect("Earthworks lock poisoned").clone();
            if snapshot.revision!=requested_revision { continue; }
            let volume=voxels.read().expect("voxel lock poisoned").clone();
            let mesh=build_mesh_with_voxels(&world,x,z,lod,Some(&snapshot),&volume);
            if result_sender.send((x,z,lod,requested_revision,mesh)).is_err() {break;}
        }
    }).map_err(|e|format!("Impossible de lancer le calcul du terrain : {e}"))?;
    Ok((job_sender,result_receiver))
}

struct GpuChunk {
    vertex:wgpu::Buffer, index:wgpu::Buffer, count:u32,
    water_vertex:wgpu::Buffer, water_index:wgpu::Buffer, water_count:u32,
    lod:u8,
}
struct Explorer {
    window:Arc<Window>,
    surface:wgpu::Surface<'static>,
    device:wgpu::Device, queue:wgpu::Queue,
    config:wgpu::SurfaceConfiguration,
    pipeline:wgpu::RenderPipeline,
    water_pipeline:wgpu::RenderPipeline,
    bind_group:wgpu::BindGroup,
    uniform_buffer:wgpu::Buffer,
    depth:wgpu::TextureView,
    world:World, camera:Camera, keys:Keys,
    chunks:HashMap<(i32,i32),GpuChunk>,
    pending_chunks:HashMap<(i32,i32),(u8,u64)>,
    earth:Arc<RwLock<Earthworks>>,
    voxels:Arc<RwLock<VoxelWorld>>,
    voxel_save_path:PathBuf,
    selected:Material,
    action_status:String,
    save_path:PathBuf,
    chunk_jobs:Sender<TerrainJob>,
    chunk_results:Receiver<TerrainResult>,
    cursor_locked:bool, last_frame:Instant, title_tick:Instant,
    rendered_frames:u32,
    seed:u64,
    adapter_name:String,
}
impl Explorer {
    async fn new(window:Arc<Window>,seed:u64)->Result<Self,String> {
        let size=window.inner_size();
        let instance=wgpu::Instance::default();
        let surface=instance.create_surface(window.clone()).map_err(|e|format!("Surface GPU: {e}"))?;
        let adapter=instance.request_adapter(&wgpu::RequestAdapterOptions{
            power_preference:wgpu::PowerPreference::HighPerformance,
            compatible_surface:Some(&surface),
            force_fallback_adapter:false,
        }).await.ok_or("Aucun adaptateur graphique compatible Vulkan, DirectX 12 ou Metal trouvé.")?;
        let adapter_name=adapter.get_info().name;
        let (device,queue)=adapter.request_device(&wgpu::DeviceDescriptor{
            label:Some("AION PRIMORDIA GPU"),
            required_features:wgpu::Features::empty(),
            required_limits:wgpu::Limits::default(),
        },None).await.map_err(|e|format!("Initialisation GPU : {e}"))?;
        let caps=surface.get_capabilities(&adapter);
        let format=caps.formats.iter().find(|f|f.is_srgb()).copied().unwrap_or(caps.formats[0]);
        let present_mode=if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {wgpu::PresentMode::Fifo}else{caps.present_modes[0]};
        let mut config=wgpu::SurfaceConfiguration{
            usage:wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,width:size.width.max(1),height:size.height.max(1),
            present_mode,
            alpha_mode:caps.alpha_modes[0],
            view_formats:vec![],
            desired_maximum_frame_latency:2,
        };
        surface.configure(&device,&config);
        let shader=device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label:Some("PRIMORDIA terrain sunlight + fog"),
            source:wgpu::ShaderSource::Wgsl(include_str!("terrain.wgsl").into()),
        });
        let uniforms=Uniforms {vp:Mat4::IDENTITY.to_cols_array_2d(),eye:[0.;4],sun:[0.35,0.85,-0.3,0.]};
        let uniform_buffer=device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label:Some("View-projection and sky"),
            contents:bytemuck::bytes_of(&uniforms),
            usage:wgpu::BufferUsages::UNIFORM|wgpu::BufferUsages::COPY_DST,
        });
        let bind_layout=device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label:Some("Camera uniform layout"),entries:&[wgpu::BindGroupLayoutEntry{
                binding:0,visibility:wgpu::ShaderStages::VERTEX_FRAGMENT,ty:wgpu::BindingType::Buffer{
                    ty:wgpu::BufferBindingType::Uniform,has_dynamic_offset:false,min_binding_size:None,
                },count:None,
            }],
        });
        let bind_group=device.create_bind_group(&wgpu::BindGroupDescriptor{
            label:Some("Camera uniform"),layout:&bind_layout,
            entries:&[wgpu::BindGroupEntry{binding:0,resource:uniform_buffer.as_entire_binding()}],
        });
        let layout=device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label:Some("Terrain pipeline layout"),bind_group_layouts:&[&bind_layout],push_constant_ranges:&[],
        });
        let pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label:Some("Indexed 3D terrain"),layout:Some(&layout),
            vertex:wgpu::VertexState{module:&shader,entry_point:"vs_main",buffers:&[Vertex::layout()]},
            fragment:Some(wgpu::FragmentState{module:&shader,entry_point:"fs_main",targets:&[Some(wgpu::ColorTargetState{
                format,blend:None,write_mask:wgpu::ColorWrites::ALL,
            })]}),
            primitive:wgpu::PrimitiveState{
                topology:wgpu::PrimitiveTopology::TriangleList,
                strip_index_format:None,front_face:wgpu::FrontFace::Ccw,cull_mode:None,
                polygon_mode:wgpu::PolygonMode::Fill,unclipped_depth:false,conservative:false,
            },
            depth_stencil:Some(wgpu::DepthStencilState{
                format:wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled:true,depth_compare:wgpu::CompareFunction::Less,
                stencil:wgpu::StencilState::default(),bias:wgpu::DepthBiasState::default(),
            }),
            multisample:wgpu::MultisampleState::default(),multiview:None,
        });
        let water_pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label:Some("Ocean: transparent sea-level surface"),layout:Some(&layout),
            vertex:wgpu::VertexState{module:&shader,entry_point:"vs_main",buffers:&[Vertex::layout()]},
            fragment:Some(wgpu::FragmentState{module:&shader,entry_point:"fs_water",targets:&[Some(wgpu::ColorTargetState{
                format,blend:Some(wgpu::BlendState::ALPHA_BLENDING),write_mask:wgpu::ColorWrites::ALL,
            })]}),
            primitive:wgpu::PrimitiveState{topology:wgpu::PrimitiveTopology::TriangleList,
                strip_index_format:None,front_face:wgpu::FrontFace::Ccw,cull_mode:None,
                polygon_mode:wgpu::PolygonMode::Fill,unclipped_depth:false,conservative:false},
            depth_stencil:Some(wgpu::DepthStencilState{
                format:wgpu::TextureFormat::Depth24Plus,depth_write_enabled:false,
                depth_compare:wgpu::CompareFunction::Less,
                stencil:wgpu::StencilState::default(),bias:wgpu::DepthBiasState::default(),
            }),
            multisample:wgpu::MultisampleState::default(),multiview:None,
        });
        let depth=depth_buffer(&device,config.width,config.height);
        let world=World::new(seed);
        let camera=Camera::new(&world);
        config.width=size.width.max(1);
        let save_path=std::env::current_exe().map_err(|e|e.to_string())?
            .parent().ok_or("Dossier exécutable absent")?
            .join(format!("PRIMORDIA_{}_terrain.aion13",seed));
        let loaded=if save_path.exists() { Earthworks::load(&world,&save_path) } else {Ok(Earthworks::new(seed))};
        let (initial_edits,load_status)=match loaded {
            Ok(e)=>(e,"Q creuser | E déposer | 1-8 choisir matériau | P enregistrer | L charger".to_string()),
            Err(msg)=>(Earthworks::new(seed),format!("ERREUR sauvegarde: {msg} (fichier conservé)")),
        };
        let earth=Arc::new(RwLock::new(initial_edits));
        let voxel_save_path=save_path.with_extension("aionv14");
        let initial_voxels=if voxel_save_path.exists(){
            VoxelWorld::load(&world,&voxel_save_path).unwrap_or_else(|_|VoxelWorld::new(seed))
        }else{VoxelWorld::new(seed)};
        let voxels=Arc::new(RwLock::new(initial_voxels));
        let (chunk_jobs,chunk_results)=spawn_chunk_worker(seed,earth.clone(),voxels.clone())?;
        Ok(Self{window,surface,device,queue,config,pipeline,water_pipeline,bind_group,uniform_buffer,
            depth,world,camera,keys:Keys::default(),chunks:HashMap::new(),
            pending_chunks:HashMap::new(),chunk_jobs,chunk_results,earth,voxels,voxel_save_path,
            selected:Material::Soil,action_status:load_status,save_path,cursor_locked:false,
            last_frame:Instant::now(),title_tick:Instant::now(),rendered_frames:0,seed,adapter_name})
    }
    fn target_ground(&self)->(f64,f64) {
        let start=self.camera.position;
        let ray=self.camera.forward();
        let earth=self.earth.read().expect("earthworks lock poisoned");
        for i in 2..=84 {
            let t=i as f32*0.5;
            let p=start+ray*t;
            if p.y as f64<=earth.elevation_m(&self.world,p.x as f64,p.z as f64) {
                return (p.x as f64,p.z as f64);
            }
        }
        let p=start+glam::Vec3::new(ray.x,0.0,ray.z).normalize_or_zero()*8.0;
        (p.x as f64,p.z as f64)
    }
    fn changed_ground(&mut self,point:(f64,f64)) {
        let (cx,cz)=chunk_key(point.0,point.1);
        // Recreate affected meshes at the new revision, including seam neighbours.
        for dz in -1..=1 {for dx in -1..=1 {
            self.chunks.remove(&(cx+dx,cz+dz));
        }}
        // All queued jobs carry the previous global revision.
        self.pending_chunks.clear();
    }
    fn excavate(&mut self) {
        let point=self.target_ground();
        let result=self.earth.write().expect("earthworks lock poisoned").excavate(&self.world,point.0,point.1);
        match result {
            Ok(m)=>{self.action_status=format!("Extrait 2 m³ de {}",m.name());self.changed_ground(point);},
            Err(e)=>self.action_status=e,
        }
    }
    fn deposit(&mut self) {
        let point=self.target_ground();
        let result=self.earth.write().expect("earthworks lock poisoned").deposit(&self.world,point.0,point.1,self.selected);
        match result {
            Ok(())=>{self.action_status=format!("Déposé 2 m³ de {}",self.selected.name());self.changed_ground(point);},
            Err(e)=>self.action_status=e,
        }
    }
    fn voxel_raycast(&self)->Option<aion_world::RayHit>{
        let p=self.camera.position;
        let forward=self.camera.forward();
        self.voxels.read().expect("voxel lock poisoned").raycast(&self.world,
            [p.x as f64,p.y as f64,p.z as f64],
            [forward.x as f64,forward.y as f64,forward.z as f64],32.0)
    }
    fn dig_voxel(&mut self) {
        let Some(hit)=self.voxel_raycast()else{
            self.action_status="Vise un bloc de roche a moins de 32 m".into();return;
        };
        let result=self.voxels.write().expect("voxel lock poisoned").dig(&self.world,hit.solid);
        match result {
            Ok(material)=>{
                self.earth.write().expect("earthworks lock poisoned").revision+=1;
                self.changed_ground((hit.solid.center()[0],hit.solid.center()[2]));
                self.action_status=format!("Creuse 8 m³ de {} a {:?}",material.name(),hit.solid);
            },
            Err(e)=>self.action_status=e,
        }
    }
    fn place_voxel(&mut self) {
        let Some(hit)=self.voxel_raycast()else{
            self.action_status="Vise une paroi pour placer un bloc".into();return;
        };
        let Some(target)=hit.last_air else{
            self.action_status="Aucun espace libre entre toi et la roche".into();return;
        };
        let result=self.voxels.write().expect("voxel lock poisoned").place(&self.world,target,self.selected);
        match result {
            Ok(())=>{
                self.earth.write().expect("earthworks lock poisoned").revision+=1;
                self.changed_ground((target.center()[0],target.center()[2]));
                self.action_status=format!("Place 8 m³ de {} a {:?}",self.selected.name(),target);
            },
            Err(e)=>self.action_status=e,
        }
    }
    fn save_edits(&mut self) {
        let r=self.earth.read().expect("earthworks lock poisoned").save(&self.save_path);
        let v=self.voxels.read().expect("voxel lock poisoned").save(&self.voxel_save_path);
        self.action_status=match (r,v) {
            (Ok(()),Ok(()))=>format!("Terrain et cavités sauvegardés : {}",self.voxel_save_path.display()),
            (a,b)=>format!("ERREUR sauvegarde : terrain={a:?}, voxels={b:?}"),
        };
    }
    fn load_edits(&mut self) {
        let old=Earthworks::load(&self.world,&self.save_path);
        let volume=VoxelWorld::load(&self.world,&self.voxel_save_path);
        match (old,volume) {
            (Ok(e),Ok(v))=>{
                *self.earth.write().expect("earthworks lock poisoned")=e;
                *self.voxels.write().expect("voxel lock poisoned")=v;
                self.chunks.clear();self.pending_chunks.clear();
                self.action_status="Terrain 2.5D et cavités 3D restaurés, volumes vérifiés".into();
            },
            (a,b)=>self.action_status=format!("Sauvegardes non chargées : surface={a:?}, volume={b:?}"),
        }
    }
    fn resize(&mut self,width:u32,height:u32) {
        if width==0||height==0{return;}
        self.config.width=width;self.config.height=height;
        self.surface.configure(&self.device,&self.config);
        self.depth=depth_buffer(&self.device,width,height);
    }
    fn add_chunk(&mut self,x:i32,z:i32,lod:u8,mesh:Mesh) {
        let Mesh{vertices,indices}=mesh;
        let vertex=self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label:Some("Terrain LOD vertices"),contents:bytemuck::cast_slice(&vertices),usage:wgpu::BufferUsages::VERTEX,
        });
        let index=self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label:Some("Terrain LOD triangles"),contents:bytemuck::cast_slice(&indices),usage:wgpu::BufferUsages::INDEX,
        });
        let water=water_mesh(x,z);
        let water_vertex=self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label:Some("Ocean surface vertices"),contents:bytemuck::cast_slice(&water.vertices),usage:wgpu::BufferUsages::VERTEX,
        });
        let water_index=self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label:Some("Ocean surface triangles"),contents:bytemuck::cast_slice(&water.indices),usage:wgpu::BufferUsages::INDEX,
        });
        self.chunks.insert((x,z),GpuChunk{vertex,index,count:indices.len() as u32,
            water_vertex,water_index,water_count:water.indices.len() as u32,lod});
    }
    fn update(&mut self) {
        let now=Instant::now();
        let dt=now.duration_since(self.last_frame).as_secs_f32().min(0.075);
        self.last_frame=now;
        {
            let terrain=self.earth.read().expect("earthworks lock poisoned");
            self.camera.update_with_edits(&self.world,&self.keys,dt,Some(&terrain));
        }
        let (cx,cz)=chunk_key(self.camera.position.x as f64,self.camera.position.z as f64);
        self.chunks.retain(|&(x,z),_|(x-cx).abs()<=RADIUS && (z-cz).abs()<=RADIUS);
        // The heavy mesh generation runs on a background thread; never freeze
        // the Win32 event loop while sampling 2m terrain or vegetation.
        for _ in 0..2 {
            match self.chunk_results.try_recv() {
                Ok((x,z,lod,revision,mesh)) => {
                    if self.pending_chunks.get(&(x,z))==Some(&(lod,revision)) {
                        self.pending_chunks.remove(&(x,z));
                    }
                    if (x-cx).abs()<=RADIUS && (z-cz).abs()<=RADIUS &&
                        lod_at(x-cx,z-cz)==lod &&
                        self.earth.read().expect("earthworks lock poisoned").revision==revision {
                        self.add_chunk(x,z,lod,mesh);
                    }
                },
                Err(_) => break,
            }
        }
        if self.pending_chunks.len()<8 {
            if let Some((x,z))=ordered_neighbours(cx,cz,RADIUS).into_iter()
                .find(|&(x,z)|{
                    let needed=lod_at(x-cx,z-cz);
                    self.chunks.get(&(x,z)).map(|v|v.lod!=needed).unwrap_or(true)
                        && self.pending_chunks.get(&(x,z)).map(|(l,r)|*l==needed &&
                            *r==self.earth.read().expect("earthworks lock poisoned").revision).unwrap_or(false)==false
                }) {
                let lod=lod_at(x-cx,z-cz);
                let revision=self.earth.read().expect("earthworks lock poisoned").revision;
                if self.chunk_jobs.send((x,z,lod,revision)).is_ok() {
                    self.pending_chunks.insert((x,z),(lod,revision));
                }
            }
        }
        self.rendered_frames+=1;
        if now.duration_since(self.title_tick)>=Duration::from_secs(1) {
            let sample=self.world.sample(self.camera.position.x as f64,self.camera.position.z as f64);
            let earth=self.earth.read().expect("earthworks lock poisoned");
            self.window.set_title(&format!("PRIMORDIA 0.14 | {} FPS | {} zones | {:?} | {} | {} ({:.0} m³ sur sol / {:.0} m³ voxel) | {} cellules 2.5D / {} voxels 3D | Q/E surface B/N cave P/L | GPU {}",
                self.rendered_frames,self.chunks.len(),sample.biome,self.action_status,
                self.selected.name(),earth.inventory_volume_m3(self.selected),
                self.voxels.read().expect("voxel lock poisoned").inventory_units(self.selected) as f64*VOXEL_VOLUME_M3,
                earth.cell_count(),self.voxels.read().expect("voxel lock poisoned").edited_count(),self.adapter_name));
            self.rendered_frames=0;self.title_tick=now;
        }
    }
    fn render(&mut self)->Result<(),wgpu::SurfaceError> {
        let frame=self.surface.get_current_texture()?;
        let view=frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let uniform=Uniforms{
            vp:self.camera.projection_view(self.config.width as f32/self.config.height as f32).to_cols_array_2d(),
            eye:[self.camera.position.x,self.camera.position.y,self.camera.position.z,1.0],
            sun:[0.33,0.86,-0.38,0.],
        };
        self.queue.write_buffer(&self.uniform_buffer,0,bytemuck::bytes_of(&uniform));
        let mut encoder=self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label:Some("Frame")});
        {
            let mut pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label:Some("Terrain + distance fog"),
                color_attachments:&[Some(wgpu::RenderPassColorAttachment{
                    view:&view,resolve_target:None,
                    ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color{r:0.45,g:0.64,b:0.78,a:1.0}),store:wgpu::StoreOp::Store},
                })],
                depth_stencil_attachment:Some(wgpu::RenderPassDepthStencilAttachment{
                    view:&self.depth,
                    depth_ops:Some(wgpu::Operations{load:wgpu::LoadOp::Clear(1.0),store:wgpu::StoreOp::Store}),
                    stencil_ops:None,
                }),
                timestamp_writes:None,occlusion_query_set:None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0,&self.bind_group,&[]);
            for chunk in self.chunks.values(){
                pass.set_vertex_buffer(0,chunk.vertex.slice(..));
                pass.set_index_buffer(chunk.index.slice(..),wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..chunk.count,0,0..1);
            }
            pass.set_pipeline(&self.water_pipeline);
            for chunk in self.chunks.values(){
                pass.set_vertex_buffer(0,chunk.water_vertex.slice(..));
                pass.set_index_buffer(chunk.water_index.slice(..),wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..chunk.water_count,0,0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();Ok(())
    }
    fn grab(&mut self,enabled:bool) {
        self.cursor_locked=enabled;
        self.window.set_cursor_visible(!enabled);
        if enabled {
            let _=self.window.set_cursor_grab(CursorGrabMode::Locked).or_else(|_|
                self.window.set_cursor_grab(CursorGrabMode::Confined));
        }else{let _=self.window.set_cursor_grab(CursorGrabMode::None);}
    }
}
fn depth_buffer(device:&wgpu::Device,width:u32,height:u32)->wgpu::TextureView {
    let texture=device.create_texture(&wgpu::TextureDescriptor{
        label:Some("Depth buffer"),
        size:wgpu::Extent3d{width,height,depth_or_array_layers:1},
        mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,
        format:wgpu::TextureFormat::Depth24Plus,usage:wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats:&[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
fn set_key(k:&mut Keys,key:KeyCode,pressed:bool) {
    match key {
        KeyCode::KeyW|KeyCode::ArrowUp=>k.forward=pressed,
        KeyCode::KeyS|KeyCode::ArrowDown=>k.backward=pressed,
        KeyCode::KeyA|KeyCode::ArrowLeft=>k.left=pressed,
        KeyCode::KeyD|KeyCode::ArrowRight=>k.right=pressed,
        KeyCode::ShiftLeft|KeyCode::ShiftRight=>k.sprint=pressed,
        KeyCode::Space=>k.up=pressed,
        KeyCode::ControlLeft=>k.down=pressed,
        _=>(),
    }
}

#[cfg(windows)]
fn show_error(message:&str) {
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;
    #[link(name="user32")]
    extern "system" {fn MessageBoxW(hwnd:isize,text:*const u16,caption:*const u16,flags:u32)->i32;}
    let s=OsStr::new(message).encode_wide().chain(Some(0)).collect::<Vec<_>>();
    let title=OsStr::new("PRIMORDIA - erreur GPU").encode_wide().chain(Some(0)).collect::<Vec<_>>();
    unsafe{MessageBoxW(0,s.as_ptr(),title.as_ptr(),0x10);}
}
#[cfg(not(windows))]
fn show_error(message:&str){eprintln!("{message}");}

fn self_test()->Result<(),String> {
    let w=World::new(DEFAULT_SEED);
    for &(x,z) in &[(-2,3),(0,0),(2,-1)] {
        let m=build_mesh_lod(&w,x,z,2);
        if m.vertices.len()<1089||m.indices.len()<6144{return Err("Invalid world chunk mesh".into())}
        if m.indices.iter().any(|&i|i>=m.vertices.len() as u32) {return Err("Index out of bounds".into())}
    }
    let mut c=Camera::new(&w);
    let keys=Keys{forward:true,..Default::default()};
    for _ in 0..100 {c.update(&w,&keys,0.016);}
    if !c.position.is_finite() {return Err("Nonfinite camera".into())}
    let mut earth=Earthworks::new(w.seed);
    let mat=earth.excavate(&w,0.0,0.0)?;
    earth.deposit(&w,4.0,0.0,mat)?;
    if !earth.material_balance_ok(){return Err("Earthworks conservation failed".into());}
    let m=build_mesh_lod_edited(&w,0,0,0,Some(&earth));
    if !m.vertices.iter().all(|v|v.position[1].is_finite()) {return Err("Earthworks mesh nonfinite".into());}
    let mut voxels=VoxelWorld::new(w.seed);
    let x=11.0;let z=15.0;let y=((w.elevation_m(x,z)-1.0)/2.0).floor()*2.0+1.0;
    let key=aion_world::VoxelKey::at(x,y,z).ok_or("Voxel index")?;
    voxels.dig(&w,key)?;
    let cave=build_mesh_with_voxels(&w,0,0,0,None,&voxels);
    if !voxels.material_balance_ok()||cave.indices.iter().any(|&n|n as usize>=cave.vertices.len()){
        return Err("Volumetric rock mesh invalid".into());
    }
    Ok(())
}

fn main() {
    #[cfg(windows)]
    std::panic::set_hook(Box::new(|info| {
        let message=format!("PRIMORDIA a rencontré une erreur interne :\n\n{info}\n\nDétails enregistrés dans le dossier temporaire.");
        let _=std::fs::write(std::env::temp_dir().join("primordia-startup-error.txt"), &message);
        show_error(&message);
    }));
    if let Err(e)=run() {
        let _=std::fs::write(std::env::temp_dir().join("primordia-startup-error.txt"), &e);
        show_error(&e);
        std::process::exit(1)
    }
}
fn run()->Result<(),String> {
    let args:Vec<String>=std::env::args().collect();
    if args.iter().any(|x|x=="--self-test") {return self_test();}
    if args.iter().any(|x|x=="--window-smoke") {
        // Real native window creation, but no GPU adapter required on CI runners.
        let loop_=EventLoop::new().map_err(|e|e.to_string())?;
        let window=WindowBuilder::new().with_title("PRIMORDIA Window Smoke Test")
            .with_inner_size(winit::dpi::LogicalSize::new(640.,480.))
            .build(&loop_).map_err(|e|format!("Window creation failed: {e}"))?;
        if window.inner_size().width==0 {return Err("Window has zero size".to_string());}
        return Ok(());
    }
    let seed=args.windows(2).find(|pair|pair[0]=="--seed")
        .map(|pair|pair[1].parse::<u64>().map_err(|_|"Usage: --seed <u64>".to_string()))
        .transpose()?.unwrap_or(DEFAULT_SEED);
    let event_loop=EventLoop::new().map_err(|e|e.to_string())?;
    let window=Arc::new(WindowBuilder::new().with_title("PRIMORDIA 0.14 - Sous-sol 3D et galeries")
        .with_inner_size(winit::dpi::LogicalSize::new(1360.,820.))
        .build(&event_loop).map_err(|e|e.to_string())?);
    let mut app=pollster::block_on(Explorer::new(window.clone(),seed))?;
    let mut next_frame=Instant::now();
    event_loop.run(move |event,target|{
        target.set_control_flow(ControlFlow::WaitUntil(next_frame));
        match event {
            Event::DeviceEvent{event:DeviceEvent::MouseMotion{delta:(dx,dy)},..} => {
                if app.cursor_locked {app.camera.look(dx,dy)}
            },
            Event::WindowEvent{window_id,event} if window_id==window.id()=>match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(size) => app.resize(size.width,size.height),
                WindowEvent::MouseInput{button:MouseButton::Left,state:ElementState::Pressed,..} => app.grab(true),
                WindowEvent::Focused(false) => {app.grab(false);app.keys=Keys::default();},
                WindowEvent::KeyboardInput{event,..}=> {
                    let pressed=event.state==ElementState::Pressed;
                    if let PhysicalKey::Code(code)=event.physical_key {
                        set_key(&mut app.keys,code,pressed);
                        if pressed&&!event.repeat {match code {
                            KeyCode::Escape=>app.grab(false),
                            KeyCode::KeyF=>app.camera.toggle_flight(),
                            KeyCode::KeyQ=>app.excavate(),
                            KeyCode::KeyB=>app.dig_voxel(),
                            KeyCode::KeyN=>app.place_voxel(),
                            KeyCode::KeyE=>app.deposit(),
                            KeyCode::KeyP=>app.save_edits(),
                            KeyCode::KeyL=>app.load_edits(),
                            KeyCode::Digit1=>app.selected=Material::Soil,
                            KeyCode::Digit2=>app.selected=Material::Sand,
                            KeyCode::Digit3=>app.selected=Material::Clay,
                            KeyCode::Digit4=>app.selected=Material::Gravel,
                            KeyCode::Digit5=>app.selected=Material::Peat,
                            KeyCode::Digit6=>app.selected=Material::Limestone,
                            KeyCode::Digit7=>app.selected=Material::Granite,
                            KeyCode::Digit8=>app.selected=Material::Basalt,
                            KeyCode::Space=>app.camera.jump(),
                            KeyCode::KeyR=>{app.camera=Camera::new(&app.world);},
                            _=>(),
                        }}
                    }
                },
                WindowEvent::RedrawRequested=>{
                    app.update();
                    match app.render(){
                        Ok(())=>(),
                        Err(wgpu::SurfaceError::Lost|wgpu::SurfaceError::Outdated)=>app.resize(app.config.width,app.config.height),
                        Err(wgpu::SurfaceError::OutOfMemory)=>target.exit(),
                        Err(wgpu::SurfaceError::Timeout)=>(),
                    }
                },
                _=>(),
            },
            Event::AboutToWait=>{
                if Instant::now()>=next_frame {
                    window.request_redraw();
                    next_frame=Instant::now()+Duration::from_millis(16);
                }
            },
            _=>(),
        }
    }).map_err(|e|e.to_string())
}