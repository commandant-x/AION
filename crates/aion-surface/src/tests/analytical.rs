use super::*;

/// Hydrostatic lake-at-rest reference: the free surface is level even when
/// bed elevations differ. A diffusive-wave solver must generate zero flux.
#[test]
fn lake_at_rest_over_irregular_bed_is_stationary() {
    let beds = vec![0.0, 0.4, 0.9, 0.2, 0.6, 0.1, 0.8, 0.3, 0.7];
    let mut s = grid(beds.clone(), 3, 0.08);
    let level = 1.25_f64;
    let area = s.cell_side_m.powi(2);
    for (i, bed) in beds.iter().enumerate() {
        s.add_water(i, (level - bed) * area).unwrap();
    }
    let before: Vec<f64> = s.cells.iter().map(|c| c.water_m3).collect();
    for _ in 0..200 { s.step(0.25, 0.0).unwrap(); }
    for (cell, initial) in s.cells.iter().zip(before) {
        assert!((cell.water_m3 - initial).abs() < 1e-11);
    }
    assert!(s.residual_m3().abs() < 1e-10);
}

/// Uniform rainfall on a sealed horizontal domain has a known exact solution:
/// V(t) = V(0) + rain_rate * horizontal_area * t.
#[test]
fn uniform_rain_matches_analytical_storage() {
    let mut s = grid(vec![0.0; 12], 4, 0.1);
    let rate = 2.5e-6;
    let dt = 0.5;
    for _ in 0..120 { s.step(dt, rate).unwrap(); }
    let expected = 12.0 * 4.0 * rate * 60.0;
    assert!((s.water_m3() - expected).abs() < 1e-11);
    assert!((s.report().rain_m3 - expected).abs() < 1e-11);
    assert!(s.residual_m3().abs() < 1e-11);
}
