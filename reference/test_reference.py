"""Independent small checks of the numerical invariants planned for AION CORE.
These are NOT Rust tests; Rust compilation must still be done on a Rust machine.
"""
from math import exp, isclose, sqrt
from dataclasses import dataclass

MASK=(1<<64)-1

def splitmix64(seed, count):
    state=seed
    out=[]
    for _ in range(count):
        state=(state+0x9E3779B97F4A7C15)&MASK
        z=state
        z=((z^(z>>30))*0xBF58476D1CE4E5B9)&MASK
        z=((z^(z>>27))*0x94D049BB133111EB)&MASK
        out.append(z^(z>>31))
    return out

def test_splitmix64_golden():
    # Stable published SplitMix64 stream for seed 0.
    assert splitmix64(0,3)==[0xe220a8397b1dcdaf,0x6e789e6aa1b965f4,0x06c45d188009454f]
    assert splitmix64(0,5)==splitmix64(0,5)
    assert splitmix64(1,5)!=splitmix64(0,5)


def conservative_heat_exchange(ca,cb,ta,tb,conductance,dt):
    q=conductance*(ta-tb)*dt
    max_q=abs(ta-tb)*ca*cb/(ca+cb)
    q=max(-max_q,min(max_q,q))
    return ta-q/ca,tb+q/cb


def test_heat_flow_and_energy_conservation():
    ca,cb=451.4,245.0
    a,b=900.,200.
    e0=a*ca+b*cb
    for _ in range(10000):
        a,b=conservative_heat_exchange(ca,cb,a,b,800.,.005)
        assert a>=0 and b>=0
    assert abs(a-b)<1e-9
    assert isclose(e0,a*ca+b*cb,rel_tol=0,abs_tol=1e-6)


def test_two_body_impulse_conservation():
    m1,m2=3.,5.
    v1,v2=4.,-2.
    e=0.25
    momentum0=m1*v1+m2*v2
    kinetic0=0.5*m1*v1*v1+0.5*m2*v2*v2
    inv1,inv2=1/m1,1/m2
    j=-(1+e)*(v2-v1)/(inv1+inv2)
    v1-=j*inv1
    v2+=j*inv2
    kinetic1=0.5*m1*v1*v1+0.5*m2*v2*v2
    assert isclose(momentum0,m1*v1+m2*v2,abs_tol=1e-12)
    assert kinetic1<kinetic0
    assert isclose(kinetic0,kinetic1+(kinetic0-kinetic1),abs_tol=1e-12)


def test_spring_energy_conversion_on_fracture():
    stiffness=100.
    ext=2.
    heat_before=35000.
    total_before=heat_before+.5*stiffness*ext**2
    released=.5*stiffness*ext**2
    total_after=heat_before+released
    assert isclose(total_before,total_after)


def test_reaction_stoichiometry_and_energy():
    # 2 H2 + O2 -> 2 H2O; H, O number of atoms conserved.
    h2,o2,h2o=4.,2.,0.
    heat_capacity=1e6
    temp=500.
    enthalpy_water=-285830.
    heat0=heat_capacity*temp+enthalpy_water*h2o
    atoms_h0=2*h2+2*h2o
    atoms_o0=2*o2+h2o
    for _ in range(500):
        limit=min(h2/2,o2)
        rate=2*exp(-1000/(8.314462618*temp))
        extent=min(rate*limit*0.01,limit)
        before=enthalpy_water*h2o
        h2-=2*extent
        o2-=extent
        h2o+=2*extent
        temp-=(enthalpy_water*h2o-before)/heat_capacity
        assert min(h2,o2,h2o)>=-1e-12
    assert isclose(2*h2+2*h2o,atoms_h0,abs_tol=1e-10)
    assert isclose(2*o2+h2o,atoms_o0,abs_tol=1e-10)
    assert isclose(temp*heat_capacity+enthalpy_water*h2o,heat0,abs_tol=1e-5)


def test_policy_gradient_moves_without_recipe():
    weights=[[0.,0.],[0.,0.],[0.,0.]]
    bias=[0.,0.,0.]
    obs=[1.,0.]
    def probs():
        a=[bias[i]+sum(w*x for w,x in zip(weights[i],obs)) for i in range(3)]
        z=[exp(y-max(a)) for y in a]
        return [v/sum(z) for v in z]
    old=probs()[2]
    for _ in range(50):
        p=probs()
        for a in range(3):
            delta=0.05*(float(a==2)-p[a])
            bias[a]+=delta
            for j in range(2):weights[a][j]+=delta*obs[j]
    assert probs()[2]>old+.2


if __name__=='__main__':
    import pytest
    raise SystemExit(pytest.main([__file__,'-q']))


def test_electrostatic_attraction_repulsion_and_energy():
    k=8_987_551_792.3
    eps=.05
    def trial(sign):
        x=[-.5,.5];v=[0.,0.];q=[1e-6,sign*1e-6];m=.01
        def force():
            delta=x[1]-x[0]
            f=-k*q[0]*q[1]*delta/(delta*delta+eps*eps)**1.5
            return f,-f
        def energy():
            return .5*m*(v[0]**2+v[1]**2)+k*q[0]*q[1]/sqrt((x[1]-x[0])**2+eps**2)
        energy0=energy();p0=sum(v)
        for _ in range(1000):
            h=.0001
            f=force()
            for i in (0,1):v[i]+=h*.5*f[i]/m;x[i]+=h*v[i]
            f=force()
            for i in (0,1):v[i]+=h*.5*f[i]/m
        assert abs(sum(v)-p0)<1e-12
        assert abs(energy()-energy0)<1e-4
        return x
    away=trial(+1)
    closer=trial(-1)
    assert away[1]-away[0]>1.
    assert closer[1]-closer[0]<1.


def test_2d_wave_cfl_and_stability():
    w=h=32;c=2.;dx=1.;dt=.1
    assert c*dt/dx<=1/sqrt(2)
    p=[0.]*(w*h);u=[0.]*(w*h)
    p[(h//2)*w+w//2]=1.;u[(h//2)*w+w//2]=1.
    for _ in range(1000):
        nxt=[0.]*len(u);alpha=(c*dt/dx)**2
        for y in range(h):
            for x in range(w):
                i=y*w+x
                left=y*w+(x-1)%w;right=y*w+(x+1)%w
                up=((y-1)%h)*w+x;down=((y+1)%h)*w+x
                nxt[i]=2*u[i]-p[i]+alpha*(u[left]+u[right]+u[up]+u[down]-4*u[i])
        p,u=u,nxt
        assert max(map(abs,u))<5
    assert max(map(abs,u))>0
