//! Axial hex coordinates (pointy-top), matching the web build.
//!
//! World mapping is in metres, derived from flat-to-flat `ff`:
//! circumradius `R = ff / sqrt(3)`, so `x = ff * (q + r/2)`, `z = R * 1.5 * r`,
//! and a standard 32 m hex has area ~886 m² (the configurator's figure).

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Axial {
    pub q: i32,
    pub r: i32,
}

impl Axial {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
    pub fn key(&self) -> String {
        format!("{},{}", self.q, self.r)
    }
    pub fn s(&self) -> i32 {
        -self.q - self.r
    }
    /// The 6 face-neighbour directions, starting east and going counter-clockwise.
    pub fn neighbor(&self, dir: usize) -> Self {
        let (dq, dr) = DIRS[dir % 6];
        Self::new(self.q + dq, self.r + dr)
    }
    pub fn neighbors(&self) -> [Self; 6] {
        std::array::from_fn(|i| self.neighbor(i))
    }
    pub fn distance(&self, other: &Self) -> i32 {
        let dq = self.q - other.q;
        let dr = self.r - other.r;
        let ds = self.s() - other.s();
        (dq.abs() + dr.abs() + ds.abs()) / 2
    }
    /// All cells at exactly `radius` steps (a ring); radius 0 is the cell itself.
    pub fn ring(&self, radius: u32) -> Vec<Self> {
        if radius == 0 {
            return vec![*self];
        }
        let mut out = Vec::with_capacity((radius * 6) as usize);
        // Start at dir 4 * radius, walk the 6 edges.
        let mut cur = Self::new(
            self.q + DIRS[4].0 * radius as i32,
            self.r + DIRS[4].1 * radius as i32,
        );
        for i in 0..6 {
            for _ in 0..radius {
                out.push(cur);
                cur = cur.neighbor(i);
            }
        }
        out
    }
    /// All cells within `radius` steps, centre first.
    pub fn area(&self, radius: u32) -> Vec<Self> {
        let mut out = vec![*self];
        for n in 1..=radius {
            out.extend(self.ring(n));
        }
        out
    }
    /// Centre of this hex in world metres for a given flat-to-flat size.
    pub fn to_world(&self, flat_to_flat_m: f64) -> (f64, f64) {
        let r_circum = flat_to_flat_m / 3f64.sqrt();
        let x = flat_to_flat_m * (self.q as f64 + self.r as f64 / 2.0);
        let z = r_circum * 1.5 * self.r as f64;
        (x, z)
    }
    /// The 6 vertex (corner) world positions, pointy-top (angle = 60i − 30°),
    /// at the true circumradius so adjacent cells share vertices exactly.
    /// (The web configurator used a 0.94 visual inset for its interlock
    /// markers, which is why its thruster count read 0 — shrunk corners
    /// never coincide. Vertex identity here uses the real corners.)
    pub fn vertices_world(&self, flat_to_flat_m: f64) -> [(f64, f64); 6] {
        let (cx, cz) = self.to_world(flat_to_flat_m);
        let r_circum = flat_to_flat_m / 3f64.sqrt();
        std::array::from_fn(|i| {
            let a = (std::f64::consts::PI / 3.0) * i as f64 - std::f64::consts::PI / 6.0;
            (cx + r_circum * a.cos(), cz + r_circum * a.sin())
        })
    }
}

pub const DIRS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

impl fmt::Display for Axial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.q, self.r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_and_distance() {
        let c = Axial::new(0, 0);
        assert_eq!(c.ring(1).len(), 6);
        assert_eq!(c.area(2).len(), 19);
        assert_eq!(c.distance(&Axial::new(2, -1)), 2);
    }

    #[test]
    fn standard_hex_area_geometry() {
        // Neighbour centres of a 32 m flat-to-flat hex sit 32 m apart in x for (1,0).
        let (x, z) = Axial::new(1, 0).to_world(32.0);
        assert!((x - 32.0).abs() < 1e-9);
        assert!(z.abs() < 1e-9);
    }
}
