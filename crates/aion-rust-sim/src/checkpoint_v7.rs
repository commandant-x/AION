//! Pure Rust counterpart to `native/checkpoint_portable.c`.
//! The *codec* is platform-neutral; this module still awaits cargo validation in CI.
//! The wire format is not a native Rust/C struct dump.
use super::{AionSim, Edge, Node, V3, MAX_BONDS, MAX_NODES};

const HEADER: usize = 32;
const FIXED: usize = 100;
const NODE_BYTES: usize = 88;
const EDGE_BYTES: usize = 100;
const MAGIC: &[u8; 8] = b"AIONS007";

fn hash(bytes: &[u8]) -> u64 {
    let mut h = 1469598103934665603u64;
    for &b in bytes { h = (h ^ b as u64).wrapping_mul(1099511628211); }
    h
}

fn write_i(b: &mut Vec<u8>, n: i32) { b.extend_from_slice(&n.to_le_bytes()); }
fn write_f(b: &mut Vec<u8>, n: f64) { b.extend_from_slice(&n.to_bits().to_le_bytes()); }

struct Reader<'a> { bytes: &'a [u8], offset: usize }
impl Reader<'_> {
    fn i(&mut self) -> Option<i32> {
        let slice = self.bytes.get(self.offset..self.offset.checked_add(4)?)?;
        let mut a = [0u8; 4]; a.copy_from_slice(slice); self.offset += 4;
        Some(i32::from_le_bytes(a))
    }
    fn f(&mut self) -> Option<f64> {
        let slice = self.bytes.get(self.offset..self.offset.checked_add(8)?)?;
        let mut a = [0u8; 8]; a.copy_from_slice(slice); self.offset += 8;
        Some(f64::from_bits(u64::from_le_bytes(a)))
    }
}

fn valid(s: &AionSim) -> bool {
    let (n, e) = (s.node_count, s.edge_count);
    if n < 0 || n as usize > MAX_NODES || e < 0 || e as usize > MAX_BONDS { return false }
    if s.material_mode < 0 || s.material_mode > 2 || !(0..=1).contains(&s.floor_enabled) { return false }
    if [s.fractures,s.contacts,s.rejected_steps,s.plastic_events,s.fatigue_fractures].iter().any(|x| *x < 0) { return false }
    let scalars = [s.sim_time,s.gravity,s.max_strain,s.max_contact_impulse,s.plastic_heat,s.damage_heat,s.damping_heat,s.max_damage];
    if scalars.iter().any(|x| (!x.is_finite() || x.abs() >= 1.e300)) || s.sim_time < 0. || s.max_damage > 1. ||
       [s.max_strain,s.max_contact_impulse,s.plastic_heat,s.damage_heat,s.damping_heat,s.max_damage].iter().any(|x| *x < 0.) { return false }
    for p in s.nodes.iter().take(n as usize) {
        let vals=[p.p.x,p.p.y,p.p.z,p.v.x,p.v.y,p.v.z,p.r,p.m,p.t,p.cv];
        if vals.iter().any(|v| (!v.is_finite() || v.abs() >= 1.e300)) || p.r <= 0. || p.m <= 0. || p.t < 0. || p.cv <= 0. ||
           p.component < 0 || p.component >= n || !(0..=3).contains(&p.kind) { return false }
    }
    for b in s.edges.iter().take(e as usize) {
        let vals=[b.rest,b.k,b.limit,b.conductance,b.original_rest,b.yield_strain,b.plastic_rate,b.damage,b.damage_onset,b.damage_rate,b.dashpot];
        if vals.iter().any(|v| (!v.is_finite() || v.abs() >= 1.e300)) || b.a < 0 || b.a >= n || b.b < 0 || b.b >= n || b.a == b.b || !(0..=1).contains(&b.alive) { return false }
        if b.rest <= 0. || b.original_rest <= 0. || b.damage > 1. || vals.iter().skip(1).any(|v| *v < 0.) { return false }
    }
    true
}

impl AionSim {
    /// Serialize only active items in a stable little-endian, binary64 record.
    pub fn encode_v7(&self) -> Option<Vec<u8>> {
        if !valid(self) { return None }
        let len = HEADER + FIXED + self.node_count as usize * NODE_BYTES + self.edge_count as usize * EDGE_BYTES;
        let mut out = Vec::with_capacity(len);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&7u32.to_le_bytes());
        out.extend_from_slice(&(len as u32).to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());
        out.extend_from_slice(&0x01020304u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        for x in [self.node_count,self.edge_count,self.fractures,self.contacts,self.material_mode,self.rejected_steps,self.plastic_events,self.fatigue_fractures,self.floor_enabled] {write_i(&mut out,x)}
        for x in [self.sim_time,self.gravity,self.max_strain,self.max_contact_impulse,self.plastic_heat,self.damage_heat,self.damping_heat,self.max_damage] {write_f(&mut out,x)}
        for n in self.nodes.iter().take(self.node_count as usize) {
            for f in [n.p.x,n.p.y,n.p.z,n.v.x,n.v.y,n.v.z,n.r,n.m,n.t,n.cv] {write_f(&mut out,f)}
            write_i(&mut out,n.kind);write_i(&mut out,n.component);
        }
        for e in self.edges.iter().take(self.edge_count as usize) {
            for i in [e.a,e.b,e.alive] {write_i(&mut out,i)}
            for f in [e.rest,e.k,e.limit,e.conductance,e.original_rest,e.yield_strain,e.plastic_rate,e.damage,e.damage_onset,e.damage_rate,e.dashpot] {write_f(&mut out,f)}
        }
        if out.len()!=len {return None}
        let h = hash(&out[HEADER..]);out[16..24].copy_from_slice(&h.to_le_bytes());
        Some(out)
    }

    /// Read untrusted checkpoint into a new state; never partially mutate an existing world.
    pub fn decode_v7(buf: &[u8]) -> Option<Self> {
        if buf.len() < HEADER+FIXED || buf.len() > HEADER+FIXED+MAX_NODES*NODE_BYTES+MAX_BONDS*EDGE_BYTES {return None}
        if buf.get(0..8)? != MAGIC.as_slice() || u32::from_le_bytes(buf.get(8..12)?.try_into().ok()?) != 7 {return None}
        let len = u32::from_le_bytes(buf.get(12..16)?.try_into().ok()?) as usize;
        if len != buf.len() || u32::from_le_bytes(buf.get(24..28)?.try_into().ok()?) != 0x01020304 || buf[28..32] != [0u8;4] {return None}
        if hash(&buf[HEADER..]) != u64::from_le_bytes(buf.get(16..24)?.try_into().ok()?) {return None}
        let mut r = Reader { bytes: buf,offset: HEADER };
        let n=r.i()?;let e=r.i()?;
        if n<0 || n as usize>MAX_NODES || e<0 || e as usize>MAX_BONDS ||
           len!=HEADER+FIXED+n as usize*NODE_BYTES+e as usize*EDGE_BYTES {return None}
        let mut s=Self::default();s.node_count=n;s.edge_count=e;
        s.fractures=r.i()?;s.contacts=r.i()?;s.material_mode=r.i()?;s.rejected_steps=r.i()?;
        s.plastic_events=r.i()?;s.fatigue_fractures=r.i()?;s.floor_enabled=r.i()?;
        s.sim_time=r.f()?;s.gravity=r.f()?;s.max_strain=r.f()?;s.max_contact_impulse=r.f()?;
        s.plastic_heat=r.f()?;s.damage_heat=r.f()?;s.damping_heat=r.f()?;s.max_damage=r.f()?;
        for i in 0..n as usize {
            let p=V3::new(r.f()?,r.f()?,r.f()?);let v=V3::new(r.f()?,r.f()?,r.f()?);
            let (radius,mass,temp,cv)=(r.f()?,r.f()?,r.f()?,r.f()?);
            let (kind,component)=(r.i()?,r.i()?);
            s.nodes[i]=Node{p,v,r:radius,m:mass,t:temp,cv,kind,component};
        }
        for i in 0..e as usize {
            let (a,b,alive)=(r.i()?,r.i()?,r.i()?);
            let (rest,k,limit,conductance)=(r.f()?,r.f()?,r.f()?,r.f()?);
            let (original_rest,yield_strain,plastic_rate,damage)=(r.f()?,r.f()?,r.f()?,r.f()?);
            let (damage_onset,damage_rate,dashpot)=(r.f()?,r.f()?,r.f()?);
            s.edges[i]=Edge {a,b,alive,rest,k,limit,conductance,original_rest,yield_strain,plastic_rate,damage,damage_onset,damage_rate,dashpot};
        }
        if r.offset != len || !valid(&s) {return None}
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::AionSim;
    #[test]
    fn reencode_c_fixture_exactly() {
        let fixture = include_bytes!("../../../tests/fixtures/sample_v7.aion7");
        let world = AionSim::decode_v7(fixture).expect("C fixture must decode");
        assert_eq!(world.node_count,73);
        assert_eq!(world.edge_count,404);
        assert_eq!(world.material_mode,1);
        assert_eq!(world.encode_v7().unwrap(),fixture);
    }
    #[test]
    fn rust_codec_roundtrip_and_rejects_damage() {
        let mut world = AionSim::laboratory(2);
        for _ in 0..200 { world.step(.003); }
        let mut bytes=world.encode_v7().expect("valid physical state");
        let loaded=AionSim::decode_v7(&bytes).unwrap();
        assert_eq!(loaded.encode_v7().unwrap(),bytes);
        bytes[250]^=1;
        assert!(AionSim::decode_v7(&bytes).is_none());
    }
}
