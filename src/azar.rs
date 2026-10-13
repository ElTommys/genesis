//! Generador xorshift: reproducible con la misma semilla.

#[derive(Debug, Clone)]
pub struct Rng(pub u64);

impl Rng {
    pub fn nuevo(semilla: u64) -> Self {
        Rng(semilla.max(1))
    }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn unif(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn normal(&mut self) -> f64 {
        let u = self.unif().max(1e-12);
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * self.unif()).cos()
    }
    pub fn rango(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}
