//! A minimal random-initialized policy gradient learner, for controlled tests.
//! It has no preloaded vocabulary, recipes, world facts, or evolutionary history.
//! IMPORTANT: This is not a human-level mind, nor is it an embodied agent yet.
use crate::rng::Rng;
#[derive(Clone,Debug)]
pub struct Brain<const N:usize,const A:usize> {
    pub weights:[[f64;N];A],
    pub bias:[f64;A],
    pub learning_rate:f64,
}
impl<const N:usize,const A:usize> Brain<N,A> {
    pub fn blank(rng:&mut Rng,learning_rate:f64)->Self {
        assert!(A>=2 && N>=1 && learning_rate>0.);
        let mut weights=[[0.;N];A];
        for a in &mut weights {for w in a { *w=rng.range(-0.01,0.01); }}
        Self{weights,bias:[0.;A],learning_rate}
    }
    pub fn probabilities(&self,observation:[f64;N])->[f64;A] {
        let mut logits=[0.;A];
        for (a,z) in logits.iter_mut().enumerate(){
            *z=self.bias[a]+self.weights[a].iter().zip(observation).map(|(w,x)|w*x).sum::<f64>();
        }
        let max=logits.iter().copied().fold(f64::NEG_INFINITY,f64::max);
        let mut total=0.;for v in &mut logits {*v=(*v-max).exp();total+=*v;}
        for v in &mut logits {*v/=total;}logits
    }
    pub fn select(&self,observation:[f64;N],rng:&mut Rng)->usize {
        let probs=self.probabilities(observation);let choice=rng.unit();let mut sum=0.;
        for (i,p) in probs.iter().enumerate(){sum+=p;if choice<sum{return i;}} A-1
    }
    pub fn update(&mut self,observation:[f64;N],action:usize,reward:f64,baseline:f64){
        assert!(action<A && reward.is_finite()&&baseline.is_finite());
        let probs=self.probabilities(observation);
        let advantage=(reward-baseline).clamp(-10.,10.);
        for (a,p) in probs.iter().enumerate(){
            let delta=self.learning_rate*advantage*((a==action) as u8 as f64-p);
            self.bias[a]=(self.bias[a]+delta).clamp(-30.,30.);
            for (j,x) in observation.iter().enumerate(){self.weights[a][j]=(self.weights[a][j]+delta*x).clamp(-30.,30.);}
        }
    }
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn uninformed_initial_policy(){let mut rng=Rng::new(2);let b=Brain::<3,4>::blank(&mut rng,0.1);let p=b.probabilities([0.,0.,0.]);for x in p {assert!((x-0.25).abs()<0.001);}}
    #[test]fn reinforcement_shifts_preference(){let mut rng=Rng::new(2);let mut b=Brain::<2,3>::blank(&mut rng,0.05);let o=[1.,0.];let before=b.probabilities(o)[2];for _ in 0..50{b.update(o,2,1.,0.);}assert!(b.probabilities(o)[2]>before+0.2);}
    #[test]fn brain_repeatability(){let mut a=Rng::new(2026);let mut b=Rng::new(2026);let x=Brain::<5,8>::blank(&mut a,0.01);let y=Brain::<5,8>::blank(&mut b,0.01);assert_eq!(x.weights,y.weights);}
}
