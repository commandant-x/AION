use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }
impl Vec3 {
    pub const ZERO: Self = Self {x:0.0, y:0.0, z:0.0};
    pub const fn new(x:f64,y:f64,z:f64)->Self { Self{x,y,z} }
    pub fn dot(self,rhs:Self)->f64 { self.x*rhs.x+self.y*rhs.y+self.z*rhs.z }
    pub fn norm2(self)->f64 { self.dot(self) }
    pub fn norm(self)->f64 { self.norm2().sqrt() }
    pub fn normalized(self)->Self {let n=self.norm(); if n>1e-12 {self/n} else {Self::new(1.,0.,0.)}}
    pub fn finite(self)->bool {self.x.is_finite() && self.y.is_finite() && self.z.is_finite()}
}
impl Add for Vec3 {type Output=Self;fn add(self,r:Self)->Self{Self::new(self.x+r.x,self.y+r.y,self.z+r.z)}}
impl Sub for Vec3 {type Output=Self;fn sub(self,r:Self)->Self{Self::new(self.x-r.x,self.y-r.y,self.z-r.z)}}
impl Mul<f64> for Vec3 {type Output=Self;fn mul(self,r:f64)->Self{Self::new(self.x*r,self.y*r,self.z*r)}}
impl Div<f64> for Vec3 {type Output=Self;fn div(self,r:f64)->Self{Self::new(self.x/r,self.y/r,self.z/r)}}
impl Neg for Vec3 {type Output=Self;fn neg(self)->Self{self*-1.0}}
impl AddAssign for Vec3 {fn add_assign(&mut self,r:Self){*self=*self+r}}
impl SubAssign for Vec3 {fn sub_assign(&mut self,r:Self){*self=*self-r}}
#[cfg(test)]mod tests {use super::*;#[test]fn vec_math(){let a=Vec3::new(3.,4.,0.);assert_eq!(a.norm(),5.);assert_eq!((a*2.)/2.,a);}}
