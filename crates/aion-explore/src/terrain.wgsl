struct Camera {
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    sun_direction: vec4<f32>,
};
@group(0) @binding(0) var<uniform> camera: Camera;

struct In {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};
@vertex
fn vs_main(input:In)->Out {
    var out:Out;
    out.world_position=input.position;
    out.position=camera.view_projection*vec4<f32>(input.position,1.0);
    out.normal=input.normal;
    out.color=input.color;
    return out;
}
@fragment
fn fs_main(input:Out)->@location(0) vec4<f32> {
    let normal=normalize(input.normal);
    let lambert=max(dot(normal,normalize(camera.sun_direction.xyz)),0.0);
    let light=0.38+0.76*lambert;
    let shaded=input.color*vec3<f32>(light,light*0.99,light*0.94);
    let distance=length(input.world_position-camera.camera_position.xyz);
    let fog=clamp((distance-280.0)/1550.0,0.0,0.94);
    let horizon=vec3<f32>(0.56,0.70,0.81);
    return vec4<f32>(mix(shaded,horizon,fog),1.0);
}