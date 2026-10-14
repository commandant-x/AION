//! Deterministic uniform-grid broad phase for spherical contact candidates.
//! It never decides whether two particles collide; the narrow phase does that.
//! For radius <= cell_size / 2, every overlapping pair is in 27 neighboring cells.
use std::collections::BTreeMap;
use crate::physics::Particle;

type Cell = (i32, i32, i32);
fn coord(x: f64, cell: f64) -> i32 {
    // Positions must remain representable in integer grid coordinates.
    (x / cell).floor().clamp(i32::MIN as f64 + 2.0, i32::MAX as f64 - 2.0) as i32
}

/// Broad-phase candidates, sorted by particle index, stable across runs.
pub fn candidate_pairs(particles: &[Particle]) -> Vec<(usize, usize)> {
    if particles.len() < 2 { return Vec::new(); }
    let max_radius = particles.iter().map(|p| p.radius).fold(0.0_f64, f64::max);
    let cell_size = (max_radius * 2.0).max(1e-9);
    let mut grid: BTreeMap<Cell, Vec<usize>> = BTreeMap::new();
    let mut cells = Vec::with_capacity(particles.len());
    for (i, p) in particles.iter().enumerate() {
        assert!(p.pos.finite() && p.radius.is_finite() && p.radius > 0.0);
        let c = (coord(p.pos.x, cell_size), coord(p.pos.y, cell_size), coord(p.pos.z, cell_size));
        grid.entry(c).or_default().push(i);
        cells.push(c);
    }
    let mut pairs = Vec::new();
    for (i, &(x,y,z)) in cells.iter().enumerate() {
        for dx in -1..=1 { for dy in -1..=1 { for dz in -1..=1 {
            if let Some(indices) = grid.get(&(x + dx, y + dy, z + dz)) {
                for &j in indices {
                    if j > i { pairs.push((i,j)); }
                }
            }
        }}}
    }
    pairs.sort_unstable();
    pairs.dedup();
    pairs
}

#[cfg(test)] mod tests {
    use super::*;
    use crate::math::Vec3;
    use crate::physics::{Particle, STONE};
    #[test] fn broadphase_does_not_miss_overlapping_pairs() {
        let mut ps = Vec::new();
        for a in 0..6 {for b in 0..6 {for c in 0..3 {
            ps.push(Particle::sphere(Vec3::new(a as f64 * 0.13 - 0.4, b as f64 * 0.17, c as f64 * 0.22), 0.085, STONE, 300.0));
        }}}
        let pairs = candidate_pairs(&ps);
        for i in 0..ps.len() { for j in (i+1)..ps.len() {
            if (ps[j].pos-ps[i].pos).norm() < ps[i].radius+ps[j].radius {
                assert!(pairs.binary_search(&(i,j)).is_ok(),"overlap {i} {j} was culled");
            }
        }}
        assert!(pairs.len() < ps.len()*(ps.len()-1)/2);
    }
    #[test] fn pairs_are_unique_and_stable() {
        let ps = (0..100).map(|i| Particle::sphere(Vec3::new(i as f64 * 0.07,0.,0.), 0.05, STONE, 290.)).collect::<Vec<_>>();
        let a=candidate_pairs(&ps);
        assert_eq!(a, candidate_pairs(&ps));
        assert!(a.windows(2).all(|p|p[0]<p[1]));
    }
}
