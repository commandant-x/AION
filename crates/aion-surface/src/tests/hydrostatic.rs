use super::*;

/// Two wet cells with a level free surface must remain stationary even if
/// their bed elevations differ. All values are SI (metres and cubic metres).
#[test]
fn unequal_beds_equal_free_surface_no_spurious_flux() {
    let mut s = grid(vec![0.0,0.5],2,0.1);
    s.add_water(0,4.0).unwrap();
    s.add_water(1,2.0).unwrap();
    let initial = s.cells.iter().map(|c|c.water_m3).collect::<Vec<_>>();
    for _ in 0..100 { s.step(0.1,0.0).unwrap(); }
    for (c,v) in s.cells.iter().zip(initial) {
        assert!((c.water_m3-v).abs()<1e-10);
    }
    assert!(s.residual_m3().abs()<1e-10);
}

/// A downhill pair can never move more water than the donor owns.
#[test]
fn finite_volume_donor_cannot_become_negative() {
    let mut s = grid(vec![20.0,0.0],2,0.01);
    s.add_water(0,1e-6).unwrap();
    for _ in 0..50 { s.step(60.0,0.0).unwrap(); }
    assert!(s.cells.iter().all(|c|c.water_m3>=0.0&&c.water_m3.is_finite()));
    assert!(s.residual_m3().abs()<1e-11);
}
