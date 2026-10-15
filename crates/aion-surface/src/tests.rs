use super::*;
fn grid(bed:Vec<f64>,width:usize,n:f64)->SurfaceWater{
 SurfaceWater::new(width,bed.len()/width,2.0,bed,n).unwrap()
}
mod mass;
mod convergence;
mod coupling;
mod analytical;
mod robustness;
mod hydrostatic;
mod coupled;
