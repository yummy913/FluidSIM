use rand::Rng;

pub struct Fluid {
    pub width: usize,
    pub height: usize,

    // Simulation parameters
    pub time: f32,
    pub diffusion: f32,
    pub viscosity: f32,

    pub injection_strength: f32,
    pub injection_radius: i32,
    pub dissipation: f32,

    spawn_x: usize,
    spawn_y: usize,

    pub density: Vec<f32>,
    pub px: Vec<f32>,
    pub py: Vec<f32>,
}

impl Fluid {
    pub fn new(time: f32, diffusion: f32, viscosity: f32) -> Self {
        let width = 200;
        let height = 150;
        let size = width * height;

        Self {
            width,
            height,

            time,
            diffusion,
            viscosity,

            injection_strength: 3.0,
            injection_radius: 3,
            dissipation: 0.995,

            spawn_x: width / 2,
            spawn_y: height / 2,

            density: vec![0.0; size],
            px: vec![0.0; size],
            py: vec![0.0; size],
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    // Continuous emitter
    fn inject(&mut self) {
        let mut rng = rand::thread_rng();

        for dy in -self.injection_radius..=self.injection_radius {
            for dx in -self.injection_radius..=self.injection_radius {
                let x = self.spawn_x as i32 + dx;
                let y = self.spawn_y as i32 + dy;

                if x > 1 && x < (self.width as i32 - 1)
                    && y > 1 && y < (self.height as i32 - 1)
                {
                    let idx = self.index(x as usize, y as usize);

                    self.density[idx] += 8.0;

                    let angle =
                        rng.gen_range(0.0..std::f32::consts::TAU);

                    self.px[idx] += angle.cos() * self.injection_strength;
                    self.py[idx] += angle.sin() * self.injection_strength;
                }
            }
        }
    }

    pub fn push_velocity(&mut self, cx: usize, cy: usize, strength: f32, radius: usize) {
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as i32 - cx as i32;
                let dy = y as i32 - cy as i32;
                let dist2 = dx*dx + dy*dy;

                if dist2 < (radius*radius) as i32 {
                    let idx = self.index(x,y);
                    let factor = strength / (dist2 as f32 + 1.0);
                    self.px[idx] += dx as f32 * factor;
                    self.py[idx] += dy as f32 * factor;
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.inject();

        // Diffuse velocity
        let mut temp_x = self.px.clone();
        let mut temp_y = self.py.clone();
        self.diffuse(&mut temp_x, &self.px, self.viscosity, self.time);
        self.diffuse(&mut temp_y, &self.py, self.viscosity, self.time);
        self.px.copy_from_slice(&temp_x);
        self.py.copy_from_slice(&temp_y);

        self.project(); // ⭐ ADD HERE

        // Advect velocity
        let vx0 = self.px.clone();
        let vy0 = self.py.clone();
        let mut new_px = self.px.clone();
        let mut new_py = self.py.clone();
        self.advect(&mut new_px, &vx0, &vx0, &vy0);
        self.advect(&mut new_py, &vy0, &vx0, &vy0);
        self.px.copy_from_slice(&new_px);
        self.py.copy_from_slice(&new_py);

        self.project(); // ⭐ ADD AGAIN

        // Advect density
        let density0 = self.density.clone();
        let mut new_density = self.density.clone();
        self.advect(&mut new_density, &density0, &self.px, &self.py);
        self.density.copy_from_slice(&new_density);

        // Fade
        for d in &mut self.density {
            *d *= self.dissipation;
        }
    }

    pub fn diffuse(&self, x: &mut [f32], x0: &[f32], diffusion: f32, time: f32) {
        let a = time * diffusion * ((self.width-2)*(self.height-2)) as f32;
        Self::linear_solver(x, x0, a, 1.0+6.0*a, self.width, self.height);
    }

    fn linear_solver(x: &mut [f32], x0: &[f32], a: f32, c: f32, width: usize, height: usize) {
        for _ in 0..20 {
            for j in 1..height-1 {
                for i in 1..width-1 {
                    let idx = i + j*width;
                    x[idx] = (x0[idx] +
                        a*(x[idx+1]+x[idx-1]+x[idx+width]+x[idx-width])) / c;
                }
            }
        }
    }

    pub fn advect(&self, d: &mut [f32], d0: &[f32], vx: &[f32], vy: &[f32]) {
        let dt0 = self.time;

        for j in 1..self.height-1 {
            for i in 1..self.width-1 {
                let idx = self.index(i,j);

                let mut x = i as f32 - dt0 * vx[idx];
                let mut y = j as f32 - dt0 * vy[idx];

                x = x.clamp(0.5, (self.width-1) as f32 - 0.5);
                y = y.clamp(0.5, (self.height-1) as f32 - 0.5);

                let i0 = x.floor() as usize;
                let i1 = i0 + 1;
                let j0 = y.floor() as usize;
                let j1 = j0 + 1;

                let s1 = x - i0 as f32;
                let s0 = 1.0 - s1;
                let t1 = y - j0 as f32;
                let t0 = 1.0 - t1;

                d[idx] =
                    s0*(t0*d0[self.index(i0,j0)] + t1*d0[self.index(i0,j1)]) +
                    s1*(t0*d0[self.index(i1,j0)] + t1*d0[self.index(i1,j1)]);
            }
        }
    }

    fn project(&mut self) {
        let mut div = vec![0.0; self.width * self.height];
        let mut p = vec![0.0; self.width * self.height];

        // Compute divergence
        for j in 1..self.height-1 {
            for i in 1..self.width-1 {
                let idx = self.index(i, j);
                div[idx] = -0.5 * (
                    self.px[self.index(i+1,j)] - self.px[self.index(i-1,j)] +
                    self.py[self.index(i,j+1)] - self.py[self.index(i,j-1)]
                ) / self.width as f32;

                p[idx] = 0.0;
            }
        }

        // Solve pressure
        for _ in 0..20 {
            for j in 1..self.height-1 {
                for i in 1..self.width-1 {
                    let idx = self.index(i,j);
                    p[idx] = (
                        div[idx] +
                        p[self.index(i-1,j)] +
                        p[self.index(i+1,j)] +
                        p[self.index(i,j-1)] +
                        p[self.index(i,j+1)]
                    ) / 4.0;
                }
            }
        }

        // Subtract gradient
        for j in 1..self.height-1 {
            for i in 1..self.width-1 {
                let idx = self.index(i,j);

                self.px[idx] -= 0.5 * (
                    p[self.index(i+1,j)] - p[self.index(i-1,j)]
                ) * self.width as f32;

                self.py[idx] -= 0.5 * (
                    p[self.index(i,j+1)] - p[self.index(i,j-1)]
                ) * self.height as f32;
            }
        }
    }

}