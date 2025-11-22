use std::ops::{Index, IndexMut};

use nalgebra::Vector2;

const G: f32 = 9.81;

#[rustfmt::skip]
const U: [Vector2<f32>; 153] = [
    Vector2::new(   0.,    -8.),
    Vector2::new(   0.,    -7.),
    Vector2::new(   0.,    -6.),
    Vector2::new(   0.,    -5.),
    Vector2::new(   0.,    -4.),
    Vector2::new(   0.,    -3.),
    Vector2::new(   0.,    -2.),
    Vector2::new(   0.,    -1.),
    Vector2::new(   0.,     0.),
    Vector2::new(   0.,     1.),
    Vector2::new(   0.,     2.),
    Vector2::new(   0.,     3.),
    Vector2::new(   0.,     4.),
    Vector2::new(   0.,     5.),
    Vector2::new(   0.,     6.),
    Vector2::new(   0.,     7.),
    Vector2::new(   0.,     8.),

    Vector2::new(   1.,    -8.),
    Vector2::new(   1.,    -7.),
    Vector2::new(   1.,    -6.),
    Vector2::new(   1.,    -5.),
    Vector2::new(   1.,    -4.),
    Vector2::new(   1.,    -3.),
    Vector2::new(   1.,    -2.),
    Vector2::new(   1.,    -1.),
    Vector2::new(   1.,     0.),
    Vector2::new(   1.,     1.),
    Vector2::new(   1.,     2.),
    Vector2::new(   1.,     3.),
    Vector2::new(   1.,     4.),
    Vector2::new(   1.,     5.),
    Vector2::new(   1.,     6.),
    Vector2::new(   1.,     7.),
    Vector2::new(   1.,     8.),

    Vector2::new(   2.,    -8.),
    Vector2::new(   2.,    -7.),
    Vector2::new(   2.,    -6.),
    Vector2::new(   2.,    -5.),
    Vector2::new(   2.,    -4.),
    Vector2::new(   2.,    -3.),
    Vector2::new(   2.,    -2.),
    Vector2::new(   2.,    -1.),
    Vector2::new(   2.,     0.),
    Vector2::new(   2.,     1.),
    Vector2::new(   2.,     2.),
    Vector2::new(   2.,     3.),
    Vector2::new(   2.,     4.),
    Vector2::new(   2.,     5.),
    Vector2::new(   2.,     6.),
    Vector2::new(   2.,     7.),
    Vector2::new(   2.,     8.),

    Vector2::new(   3.,    -8.),
    Vector2::new(   3.,    -7.),
    Vector2::new(   3.,    -6.),
    Vector2::new(   3.,    -5.),
    Vector2::new(   3.,    -4.),
    Vector2::new(   3.,    -3.),
    Vector2::new(   3.,    -2.),
    Vector2::new(   3.,    -1.),
    Vector2::new(   3.,     0.),
    Vector2::new(   3.,     1.),
    Vector2::new(   3.,     2.),
    Vector2::new(   3.,     3.),
    Vector2::new(   3.,     4.),
    Vector2::new(   3.,     5.),
    Vector2::new(   3.,     6.),
    Vector2::new(   3.,     7.),
    Vector2::new(   3.,     8.),

    Vector2::new(   4.,    -8.),
    Vector2::new(   4.,    -7.),
    Vector2::new(   4.,    -6.),
    Vector2::new(   4.,    -5.),
    Vector2::new(   4.,    -4.),
    Vector2::new(   4.,    -3.),
    Vector2::new(   4.,    -2.),
    Vector2::new(   4.,    -1.),
    Vector2::new(   4.,     0.),
    Vector2::new(   4.,     1.),
    Vector2::new(   4.,     2.),
    Vector2::new(   4.,     3.),
    Vector2::new(   4.,     4.),
    Vector2::new(   4.,     5.),
    Vector2::new(   4.,     6.),
    Vector2::new(   4.,     7.),
    Vector2::new(   4.,     8.),

    Vector2::new(   5.,    -8.),
    Vector2::new(   5.,    -7.),
    Vector2::new(   5.,    -6.),
    Vector2::new(   5.,    -5.),
    Vector2::new(   5.,    -4.),
    Vector2::new(   5.,    -3.),
    Vector2::new(   5.,    -2.),
    Vector2::new(   5.,    -1.),
    Vector2::new(   5.,     0.),
    Vector2::new(   5.,     1.),
    Vector2::new(   5.,     2.),
    Vector2::new(   5.,     3.),
    Vector2::new(   5.,     4.),
    Vector2::new(   5.,     5.),
    Vector2::new(   5.,     6.),
    Vector2::new(   5.,     7.),
    Vector2::new(   5.,     8.),

    Vector2::new(   6.,    -8.),
    Vector2::new(   6.,    -7.),
    Vector2::new(   6.,    -6.),
    Vector2::new(   6.,    -5.),
    Vector2::new(   6.,    -4.),
    Vector2::new(   6.,    -3.),
    Vector2::new(   6.,    -2.),
    Vector2::new(   6.,    -1.),
    Vector2::new(   6.,     0.),
    Vector2::new(   6.,     1.),
    Vector2::new(   6.,     2.),
    Vector2::new(   6.,     3.),
    Vector2::new(   6.,     4.),
    Vector2::new(   6.,     5.),
    Vector2::new(   6.,     6.),
    Vector2::new(   6.,     7.),
    Vector2::new(   6.,     8.),

    Vector2::new(   7.,    -8.),
    Vector2::new(   7.,    -7.),
    Vector2::new(   7.,    -6.),
    Vector2::new(   7.,    -5.),
    Vector2::new(   7.,    -4.),
    Vector2::new(   7.,    -3.),
    Vector2::new(   7.,    -2.),
    Vector2::new(   7.,    -1.),
    Vector2::new(   7.,     0.),
    Vector2::new(   7.,     1.),
    Vector2::new(   7.,     2.),
    Vector2::new(   7.,     3.),
    Vector2::new(   7.,     4.),
    Vector2::new(   7.,     5.),
    Vector2::new(   7.,     6.),
    Vector2::new(   7.,     7.),
    Vector2::new(   7.,     8.),

    Vector2::new(   8.,    -8.),
    Vector2::new(   8.,    -7.),
    Vector2::new(   8.,    -6.),
    Vector2::new(   8.,    -5.),
    Vector2::new(   8.,    -4.),
    Vector2::new(   8.,    -3.),
    Vector2::new(   8.,    -2.),
    Vector2::new(   8.,    -1.),
    Vector2::new(   8.,     0.),
    Vector2::new(   8.,     1.),
    Vector2::new(   8.,     2.),
    Vector2::new(   8.,     3.),
    Vector2::new(   8.,     4.),
    Vector2::new(   8.,     5.),
    Vector2::new(   8.,     6.),
    Vector2::new(   8.,     7.),
    Vector2::new(   8.,     8.),
];

// An action u_k is either an index into U
// or one of the two special values below
const UNINIT: u8 = u8::MAX - 1;
const TERMINAL: u8 = u8::MAX;

struct BrachistochroneMemo {
    n: usize,
    inner: Box<[(f32, u8)]>,
}

impl Index<(usize, Vector2<f32>)> for BrachistochroneMemo {
    type Output = (f32, u8);

    fn index(&self, (k, x_k): (usize, Vector2<f32>)) -> &Self::Output {
        let n = self.n;
        let x = x_k.x as usize;
        let y = x_k.y as usize;

        &self.inner[k * (n + 1) * (n + 1) + x * (n + 1) + y]
    }
}

impl IndexMut<(usize, Vector2<f32>)> for BrachistochroneMemo {
    fn index_mut(&mut self, (k, x_k): (usize, Vector2<f32>)) -> &mut Self::Output {
        let n = self.n;
        let x = x_k.x as usize;
        let y = x_k.y as usize;

        &mut self.inner[k * (n + 1) * (n + 1) + x * (n + 1) + y]
    }
}

impl BrachistochroneMemo {
    fn new(n: usize, time_horizon: usize) -> Self {
        let inner = vec![(f32::INFINITY, UNINIT); (n + 1) * (n + 1) * (time_horizon + 1)];

        Self {
            n,
            inner: inner.into(),
        }
    }
}

pub struct Brachistochrone {
    n: usize,
    time_horizon: usize,
    mu: f32,

    start: Vector2<f32>,
    end: Vector2<f32>,
    memo: BrachistochroneMemo,
}

impl Brachistochrone {
    pub fn new(n: usize, mu: f32, start: Vector2<f32>, end: Vector2<f32>) -> Brachistochrone {
        // "Tighter" (not really) lower bound for time horizon found by approximating from experimental data
        let time_horizon = match n {
            1000.. => n / 5,
            500.. => n / 4,
            50.. => n / 3,
            _ => n / 2,
        };

        Self {
            n,
            time_horizon,
            mu,

            start,
            end,
            memo: BrachistochroneMemo::new(n, time_horizon),
        }
    }

    fn cost(&self, x_k: Vector2<f32>, u: &Vector2<f32>) -> f32 {
        let x_k_scaled = x_k * self.mu;
        let x_kp1_scaled = (x_k + u) * self.mu;
        let y_start_scaled = self.start.y * self.mu;

        2. * (x_kp1_scaled - x_k_scaled).norm()
            / ((2. * G * (y_start_scaled - x_kp1_scaled.y)).sqrt()
                + (2. * G * (y_start_scaled - x_k_scaled.y)).sqrt())
    }

    pub fn solve(&mut self) {
        let bounds = 0.0..=(self.n as f32);

        self.memo[(self.time_horizon, self.end)] = (0., TERMINAL);

        for k in (0..self.time_horizon).rev() {
            for x in (0..=self.n).rev() {
                for y in (0..=self.n).rev() {
                    let x_k = Vector2::<f32>::new(x as f32, y as f32);

                    let mut min_v = f32::INFINITY;
                    let mut chosen_u = UNINIT;

                    for (u_idx, u) in U.iter().enumerate() {
                        let x_k_next = x_k + u;

                        if !bounds.contains(&x_k_next.x) || !bounds.contains(&x_k_next.y) {
                            continue;
                        }

                        let (v_next, _) = self.memo[(k + 1, x_k_next)];

                        let c = self.cost(x_k, u);
                        let v_cur = c + v_next;

                        if v_cur < min_v {
                            min_v = v_cur;
                            chosen_u = u_idx as u8;
                        }
                    }

                    self.memo[(k, x_k)] = (min_v, chosen_u);
                }
            }
        }
    }

    pub fn path_iter(&self, start: Vector2<f32>) -> impl Iterator<Item = (f32, Vector2<f32>)> {
        BrachistochronePathIterator {
            memo: &self.memo,
            current: start,
            k: 0,
        }
    }
}

struct BrachistochronePathIterator<'a> {
    memo: &'a BrachistochroneMemo,
    current: Vector2<f32>,
    k: usize,
}

impl<'a> Iterator for BrachistochronePathIterator<'a> {
    type Item = (f32, Vector2<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        let (cost, u_idx) = self.memo[(self.k, self.current)];

        if u_idx == UNINIT || u_idx == TERMINAL {
            return None;
        }

        let x_k = self.current.clone();
        let u_k = &U[u_idx as usize];

        self.current += u_k;
        self.k += 1;

        return Some((cost, x_k));
    }
}
