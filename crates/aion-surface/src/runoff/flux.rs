use super::super::SurfaceCell;
/// Manning diffusive-wave Q=width*h^(5/3)*sqrt(slope)/n [m3/s].
/// h is upstream depth above the higher of the two bed elevations.
pub fn request(cells:&[SurfaceCell],i:usize,j:usize,area:f64,
               side:f64,n:f64,dt:f64)->Option<(usize,usize,f64)>{
 let a=cells[i];let b=cells[j];
 let ea=a.bed_m+a.water_m3/area;
 let eb=b.bed_m+b.water_m3/area;
 if ea==eb{return None}
 let (from,to,up,down)=if ea>eb{(i,j,ea,eb)}else{(j,i,eb,ea)};
 let h=(up-a.bed_m.max(b.bed_m)).max(0.0);
 let flux=side*h.powf(5.0/3.0)*((up-down)/side).sqrt()*dt/n;
 // Equal free-surface levels require V = (eta_up - eta_down)*area/2.
 // This cap is exact only when both cells are wet above their beds; for a
 // dry receiver it is a conservative stability limiter, not a Riemann solver.
 let equilibrium_cap=(up-down)*area*0.5;
 let v=flux.min(equilibrium_cap).min(cells[from].water_m3);
 if v>0.0{Some((from,to,v))}else{None}
}
