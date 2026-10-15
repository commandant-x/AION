//! Transactional surface/subsurface timestep and cross-domain diagnostics.
//! No groundwater exfiltration, evaporation, sediment or dynamic terrain here.
use crate::{SurfaceWater, HydroThermal};

#[derive(Clone, Copy, Debug)]
pub struct CoupledReport {
    pub time_s: f64,
    pub surface_m3: f64,
    pub subsurface_m3: f64,
    pub rain_m3: f64,
    pub infiltration_m3: f64,
    pub total_mass_residual_m3: f64,
    pub subsurface_energy_residual_j: f64,
}

/// Owns a surface catchment and an aligned three-dimensional porous patch.
/// One timestep is all-or-nothing, including errors from either solver.
/// External water consists of initial hydro storage, explicitly added surface
/// water and rainfall. Infiltration is internal and cancels in the global budget.
#[derive(Clone, Debug)]
pub struct CoupledCatchment {
    pub surface: SurfaceWater,
    pub hydro: HydroThermal,
    initial_total_m3: f64,
    initial_surface_supply_m3: f64,
    initial_rain_m3: f64,
}
impl CoupledCatchment {
    pub fn new(surface: SurfaceWater, hydro: HydroThermal) -> Result<Self,String> {
        if surface.cell_side_m != aion_world::hydrothermal::CELL_SIDE_M
            || surface.width != hydro.dims[0] || surface.depth != hydro.dims[2]
            || hydro.dims[1] == 0 || (surface.time_s-hydro.time_s).abs()>1e-9 {
            return Err("Coupled catchment requires aligned grids and synchronized clocks".into());
        }
        let initial_total_m3=surface.water_m3()+hydro.water_m3();
        let initial_surface_supply_m3=surface.report().supplied_m3;
        let initial_rain_m3=surface.report().rain_m3;
        Ok(Self{surface,hydro,initial_total_m3,initial_surface_supply_m3,initial_rain_m3})
    }
    pub fn report(&self)->CoupledReport {
        let s=self.surface.report();
        let h=self.hydro.report();
        // Hydro injection/withdrawal from other operations is not modeled as
        // an external source here; this driver requires exclusive ownership.
        let residual=self.surface.water_m3()+self.hydro.water_m3()
            -self.initial_total_m3
            -(s.supplied_m3-self.initial_surface_supply_m3)
            -(s.rain_m3-self.initial_rain_m3)
            +h.withdrawn_water_m3;
        CoupledReport{
            time_s:s.time_s,surface_m3:s.surface_water_m3,
            subsurface_m3:h.total_water_m3,rain_m3:s.rain_m3,
            infiltration_m3:s.infiltrated_m3,
            total_mass_residual_m3:residual,
            subsurface_energy_residual_j:self.hydro.energy_residual_j(),
        }
    }
    /// Sequential operator splitting: rain -> runoff -> infiltration -> Darcy/Fourier.
    /// A time step is not a converged simultaneous multiphysics solution.
    pub fn step(&mut self,dt_s:f64,rain_m_s:f64,rain_temperature_k:f64)->Result<CoupledReport,String>{
        if !dt_s.is_finite()||dt_s<=0.0||dt_s>60.0
            ||!rain_m_s.is_finite()||rain_m_s<0.0
            ||!rain_temperature_k.is_finite()
            ||!(200.0..=500.0).contains(&rain_temperature_k) {
            return Err("Invalid coupled timestep inputs".into());
        }
        let mut next=self.clone();
        next.surface.step(dt_s,rain_m_s)?;
        next.surface.infiltrate_into(&mut next.hydro,dt_s,rain_temperature_k)?;
        next.hydro.step(dt_s)?;
        let report=next.report();
        let tolerance=1e-8*(1.0+next.initial_total_m3
            +next.surface.report().supplied_m3+next.surface.report().rain_m3);
        if !report.total_mass_residual_m3.is_finite()
            ||report.total_mass_residual_m3.abs()>tolerance
            ||!report.subsurface_energy_residual_j.is_finite()
            ||report.subsurface_energy_residual_j.abs()>1e-7*next.hydro.energy_j().abs().max(1.0) {
            return Err("Coupled timestep violates global budgets".into());
        }
        *self=next;
        Ok(report)
    }
}
