use super::super::SurfaceWater;
use super::flux::request;
impl SurfaceWater {
 /// Synchronous face requests: scale all exits from a donor by its
 /// available volume, then apply equal/opposite changes in one sweep.
 pub(super) fn route(&mut self,dt:f64){
  let area=self.cell_side_m.powi(2);
  let mut flows=Vec::new();
  let mut outgoing=vec![0.0;self.cells.len()];
  for z in 0..self.depth{for x in 0..self.width{
   let i=z*self.width+x;
   for j in [if x+1<self.width{Some(i+1)}else{None},
             if z+1<self.depth{Some(i+self.width)}else{None}].into_iter().flatten(){
    if let Some((from,to,v))=request(&self.cells,i,j,area,self.cell_side_m,self.manning_n,dt){
     outgoing[from]+=v;flows.push((from,to,v))
    }
   }
  }}
  let mut delta=vec![0.0;self.cells.len()];
  for (from,to,v) in flows{
   let scale=if outgoing[from]>self.cells[from].water_m3{
    self.cells[from].water_m3/outgoing[from]
   }else{1.0};
   delta[from]-=v*scale;delta[to]+=v*scale;
  }
  for (c,d) in self.cells.iter_mut().zip(delta){
   c.water_m3=(c.water_m3+d).max(0.0);
  }
 }
}
