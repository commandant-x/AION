//! PRIMORDIA: first native GPU terrain exploration. Not yet full simulation.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod camera;
mod terrain;

use std::{collections::HashMap, sync::Arc, time::{Duration,Instant}};
use aion_world::World;
use bytemuck::{Pod,Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow,EventLoop},
    keyboard::{KeyCode,PhysicalKey},
    window::{CursorGrabMode,WindowBuilder,Window},
};
use camera::{Camera,Keys};
use terrain::{Mesh, Vertex,build_mesh,chunk_key,ordered_neighbours};

const RADIUS: i32=3; // up to 49 chunks / ~1.8 km square resident
const DEFAULT_SEED:u64=42;

#[repr(C)]
#[derive(Clone,Copy,Pod,Zeroable)]
struct Uniforms {
    vp: [[f32;4];4],
    eye: [f32;4],
    sun: [f32;4],
}

struct GpuChunk { vertex:wgpu::Buffer, index:wgpu::Buffer, count:u32 }
struct Explorer {
    window:Arc<Window>,
    surface:wgpu::Surface<'static>,
    device:wgpu::Device, queue:wgpu::Queue,
    config:wgpu::SurfaceConfiguration,
    pipeline:wgpu::RenderPipeline,
    bind_group:wgpu::BindGroup,
    uniform_buffer:wgpu::Buffer,
    depth:wgpu::TextureView,
    world:World, camera:Camera, keys:Keys,
    chunks:HashMap<(i32,i32),GpuChunk>,
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
        let depth=depth_buffer(&device,config.width,config.height);
        let world=World::new(seed);
        let camera=Camera::new(&world);
        config.width=size.width.max(1);
        Ok(Self{window,surface,device,queue,config,pipeline,bind_group,uniform_buffer,
            depth,world,camera,keys:Keys::default(),chunks:HashMap::new(),cursor_locked:false,
            last_frame:Instant::now(),title_tick:Instant::now(),rendered_frames:0,seed,adapter_name})
    }
    fn resize(&mut self,width:u32,height:u32) {
        if width==0||height==0{return;}
        self.config.width=width;self.config.height=height;
        self.surface.configure(&self.device,&self.config);
        self.depth=depth_buffer(&self.device,width,height);
    }
    fn add_chunk(&mut self,x:i32,z:i32) {
        let source=self.world.chunk(x,z);
        let Mesh{vertices,indices}=build_mesh(&source);
        let vertex=self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label:Some("Chunk vertices"),contents:bytemuck::cast_slice(&vertices),usage:wgpu::BufferUsages::VERTEX,
        });
        let index=self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label:Some("Chunk triangles"),contents:bytemuck::cast_slice(&indices),usage:wgpu::BufferUsages::INDEX,
        });
        self.chunks.insert((x,z),GpuChunk{vertex,index,count:indices.len() as u32});
    }
    fn update(&mut self) {
        let now=Instant::now();
        let dt=now.duration_since(self.last_frame).as_secs_f32().min(0.075);
        self.last_frame=now;
        self.camera.update(&self.world,&self.keys,dt);
        let (cx,cz)=chunk_key(self.camera.position.x as f64,self.camera.position.z as f64);
        self.chunks.retain(|&(x,z),_|(x-cx).abs()<=RADIUS && (z-cz).abs()<=RADIUS);
        // Limit terrain generation to one new chunk per frame to avoid stutters.
        if let Some((x,z))=ordered_neighbours(cx,cz,RADIUS).into_iter()
            .find(|key|!self.chunks.contains_key(key)) {self.add_chunk(x,z);}
        self.rendered_frames+=1;
        if now.duration_since(self.title_tick)>=Duration::from_secs(1) {
            let sample=self.world.sample(self.camera.position.x as f64,self.camera.position.z as f64);
            self.window.set_title(&format!("PRIMORDIA 0.11 | {} FPS | {} chunks | {:?} | ({:.0},{:.0},{:.0})m | GPU: {} | W/A/S/D marcher, Shift courir, souris regarder, F voler, Esc libérer",
                self.rendered_frames,self.chunks.len(),sample.biome,
                self.camera.position.x,self.camera.position.y,self.camera.position.z,self.adapter_name));
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
        let m=build_mesh(&w.chunk(x,z));
        if m.vertices.len()<1089||m.indices.len()<6144{return Err("Invalid world chunk mesh".into())}
        if m.indices.iter().any(|&i|i>=m.vertices.len() as u32) {return Err("Index out of bounds".into())}
    }
    let mut c=Camera::new(&w);
    let keys=Keys{forward:true,..Default::default()};
    for _ in 0..100 {c.update(&w,&keys,0.016);}
    if !c.position.is_finite() {return Err("Nonfinite camera".into())}
    Ok(())
}

fn main() {
    #[cfg(windows)]
    std::panic::set_hook(Box::new(|info| {
        let message=format!("PRIMORDIA a rencontré une erreur interne :\\n\\n{info}\\n\\nDétails enregistrés dans le dossier temporaire.");
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
    let window=Arc::new(WindowBuilder::new().with_title("PRIMORDIA 0.11 - GPU Explorer")
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
                            KeyCode::KeyF=>app.camera.flying=!app.camera.flying,
                            KeyCode::KeyR=>{
                                app.world=World::new(app.seed);app.camera=Camera::new(&app.world);
                                app.chunks.clear();
                            },
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