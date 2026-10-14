//! A tiny chemical kinetics testbed: stoichiometric reactions validated against
//! elemental conservation. It is NOT a database of all chemistry on Earth.

pub const ELEMENTS: [&str;6]=["H","C","N","O","Si","Fe"];
const GAS_R:f64=8.314462618; // J/(mol K)

#[derive(Clone,Debug)]
pub struct Species {
    pub label:String,
    pub atoms:[u16;6],
    pub formation_energy:f64, // J per mol
}
#[derive(Clone,Debug)]
pub struct Reaction {
    pub consume:Vec<(usize,u16)>,
    pub produce:Vec<(usize,u16)>,
    pub frequency:f64, // s^-1: simplified effective kinetics
    pub activation_energy:f64, // J per mol
}
#[derive(Clone,Debug)]
pub struct Reactor {
    pub species:Vec<Species>,
    pub concentrations:Vec<f64>, // moles, well-mixed chamber
    pub temperature:f64, // K
    pub heat_capacity:f64, // J/K, includes vessel heat reservoir
    pub reactions:Vec<Reaction>,
}
impl Reactor {
    pub fn new(species:Vec<Species>, concentrations:Vec<f64>, temperature:f64, heat_capacity:f64)->Self {
        assert!(species.len()==concentrations.len()&&temperature>0.&&heat_capacity>0.);
        assert!(concentrations.iter().all(|v|v.is_finite()&&*v>=0.));
        Self{species,concentrations,temperature,heat_capacity,reactions:Vec::new()}
    }
    pub fn add_reaction(&mut self,r:Reaction)->Result<(),String> {
        if r.consume.is_empty()||r.produce.is_empty()||r.frequency<=0.||!r.frequency.is_finite()||
          r.activation_energy<0.||!r.activation_energy.is_finite() {return Err("invalid kinetics".into());}
        let mut balance=[0_i64;6];
        let mut seen_consume=vec![false;self.species.len()];
        let mut seen_produce=vec![false;self.species.len()];
        for (id,coef) in &r.consume {
            if *id>=self.species.len()||*coef==0 {return Err("invalid reactant".into());}
            if seen_consume[*id] {return Err("duplicate reactant".into());}
            seen_consume[*id]=true;
            for (e,c) in balance.iter_mut().enumerate(){*c+=self.species[*id].atoms[e] as i64 * *coef as i64;}
        }
        for (id,coef) in &r.produce {
            if *id>=self.species.len()||*coef==0 {return Err("invalid product".into());}
            if seen_produce[*id] {return Err("duplicate product".into());}
            seen_produce[*id]=true;
            for (e,c) in balance.iter_mut().enumerate(){*c-=self.species[*id].atoms[e] as i64 * *coef as i64;}
        }
        if balance!=[0;6]{return Err(format!("reaction violates elemental conservation: {balance:?}"));}
        self.reactions.push(r);Ok(())
    }
    pub fn total_atoms(&self)->[f64;6] {
        let mut result=[0.;6];
        for (i,s) in self.species.iter().enumerate(){for (k,c) in result.iter_mut().enumerate(){*c+=self.concentrations[i]*s.atoms[k] as f64;}}
        result
    }
    pub fn chemical_energy(&self)->f64 {
        self.species.iter().enumerate().map(|(i,s)|self.concentrations[i]*s.formation_energy).sum()
    }
    pub fn total_energy(&self)->f64 {self.heat_capacity*self.temperature+self.chemical_energy()}
    pub fn step(&mut self,dt:f64){
        assert!(dt.is_finite()&&dt>0.&&dt<=1.);
        // Operator splitting; suitable only for these non-stiff toy mechanisms.
        for r in &self.reactions {
            let mut max_extent=f64::INFINITY;
            let mut reagent_availability=1.0_f64;
            for (id,c) in &r.consume {
                let amount=self.concentrations[*id]/(*c as f64);
                max_extent=max_extent.min(amount);
                reagent_availability=reagent_availability.min(amount);
            }
            if max_extent<=0. {continue;}
            let rate=r.frequency*(-r.activation_energy/(GAS_R*self.temperature)).exp();
            let extent=(rate*reagent_availability*dt).min(max_extent);
            if extent<=0. {continue;}
            let before=self.chemical_energy();
            for (id,c) in &r.consume {self.concentrations[*id]=(self.concentrations[*id]-extent*(*c as f64)).max(0.);}
            for (id,c) in &r.produce {self.concentrations[*id]+=extent*(*c as f64);}
            let chemical_change=self.chemical_energy()-before;
            self.temperature-=chemical_change/self.heat_capacity;
            assert!(self.temperature>0.,"chemical energy exceeded chamber heat reservoir");
        }
    }
}
#[cfg(test)]mod tests {
    use super::*;
    fn reactor()->Reactor {
        let species=vec![
            Species{label:"H2".into(),atoms:[2,0,0,0,0,0],formation_energy:0.},
            Species{label:"O2".into(),atoms:[0,0,0,2,0,0],formation_energy:0.},
            Species{label:"H2O".into(),atoms:[2,0,0,1,0,0],formation_energy:-285830.},
        ];
        Reactor::new(species,vec![4.,2.,0.],500.,1e6)
    }
    #[test]fn validates_atom_balance(){let mut r=reactor();assert!(r.add_reaction(Reaction{consume:vec![(0,2),(1,1)],produce:vec![(2,2)],frequency:3.,activation_energy:0.}).is_ok());assert!(r.add_reaction(Reaction{consume:vec![(0,1)],produce:vec![(2,1)],frequency:3.,activation_energy:0.}).is_err());}
    #[test]fn conserved_atoms_and_energy(){let mut r=reactor();r.add_reaction(Reaction{consume:vec![(0,2),(1,1)],produce:vec![(2,2)],frequency:2.,activation_energy:1000.}).unwrap();let atoms=r.total_atoms();let energy=r.total_energy();for _ in 0..200{r.step(0.01);}for (a,b) in r.total_atoms().iter().zip(atoms){assert!((*a-b).abs()<1e-8);}assert!((r.total_energy()-energy).abs()<1e-5);assert!(r.temperature>500.);}
    #[test]fn no_negative_moles(){let mut r=reactor();r.add_reaction(Reaction{consume:vec![(0,2),(1,1)],produce:vec![(2,2)],frequency:200.,activation_energy:0.}).unwrap();r.step(1.);assert!(r.concentrations.iter().all(|c|*c>=0.));}
}
