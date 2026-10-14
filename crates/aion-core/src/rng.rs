//! SplitMix64: stable seeded pseudo-random generator for reproducible experiments.
//! Not cryptographically secure.
#[derive(Debug,Clone,Copy)]
pub struct Rng { state:u64 }
impl Rng {
    pub fn new(seed:u64)->Self {Self{state:seed}}
    pub fn next_u64(&mut self)->u64 {
        self.state=self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z=self.state;
        z=(z^(z>>30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z=(z^(z>>27)).wrapping_mul(0x94D049BB133111EB);
        z^(z>>31)
    }
    pub fn unit(&mut self)->f64 {((self.next_u64()>>11) as f64)*(1.0/9007199254740992.0)}
    pub fn range(&mut self,low:f64,high:f64)->f64 {low+(high-low)*self.unit()}
    pub fn state(&self)->u64 {self.state}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test]fn repeatable(){let mut a=Rng::new(42);let mut b=Rng::new(42);for _ in 0..1000 {assert_eq!(a.next_u64(),b.next_u64());}}
    #[test]fn bounded(){let mut r=Rng::new(1);for _ in 0..1000{assert!((0.0..1.0).contains(&r.unit()));}}
}
