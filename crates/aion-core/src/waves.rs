//! Finite-difference 2D scalar wave experiment: periodic boundaries, no optics shader.
//! Solves d²u/dt² = c² * laplacian(u), not the full Maxwell equations.
#[derive(Clone,Debug)]
pub struct WaveGrid {
    pub width:usize,pub height:usize,pub dx:f64,pub dt:f64,pub speed:f64,
    past:Vec<f64>,pub current:Vec<f64>,pub time:f64,
}
impl WaveGrid {
    pub fn new(width:usize,height:usize,dx:f64,dt:f64,speed:f64)->Self {
        assert!(width>=3&&height>=3&&dx>0.&&dt>0.&&speed>0.);
        assert!(dx.is_finite()&&dt.is_finite()&&speed.is_finite());
        // 2D CFL condition for standard five-point Laplacian.
        assert!(speed*dt/dx<=1./2.0_f64.sqrt(),"unstable CFL ratio");
        Self{width,height,dx,dt,speed,past:vec![0.;width*height],current:vec![0.;width*height],time:0.}
    }
    pub fn pulse(&mut self,x:usize,y:usize,value:f64){
        assert!(x<self.width&&y<self.height&&value.is_finite());
        let i=y*self.width+x;self.current[i]+=value;self.past[i]+=value;
    }
    pub fn value(&self,x:usize,y:usize)->f64 {self.current[y*self.width+x]}
    pub fn step(&mut self){
        let mut next=vec![0.;self.current.len()];
        let alpha=(self.speed*self.dt/self.dx).powi(2);
        for y in 0..self.height {for x in 0..self.width {
            let p=y*self.width+x;
            let left=y*self.width+(x+self.width-1)%self.width;
            let right=y*self.width+(x+1)%self.width;
            let up=((y+self.height-1)%self.height)*self.width+x;
            let down=((y+1)%self.height)*self.width+x;
            next[p]=2.*self.current[p]-self.past[p]+alpha*
                (self.current[left]+self.current[right]+self.current[up]+self.current[down]-4.*self.current[p]);
        }}
        self.past=std::mem::replace(&mut self.current,next);
        self.time+=self.dt;
    }
    pub fn max_amplitude(&self)->f64 {self.current.iter().map(|u|u.abs()).fold(0.,f64::max)}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test]#[should_panic(expected="unstable CFL ratio")]
    fn rejects_unstable_timestep(){let _=WaveGrid::new(32,32,1.,.5,2.);}
    #[test]fn disturbance_propagates(){let mut w=WaveGrid::new(32,32,1.,.1,2.);w.pulse(16,16,1.);for _ in 0..10{w.step();}assert!(w.value(17,16).abs()>1e-6);}
    #[test]fn stable_over_long_run(){let mut w=WaveGrid::new(32,32,1.,.1,2.);w.pulse(16,16,1.);for _ in 0..1000{w.step();assert!(w.max_amplitude().is_finite());assert!(w.max_amplitude()<5.);}}
}
