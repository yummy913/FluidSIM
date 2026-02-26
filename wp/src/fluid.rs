use crate::emitter::Emitter;
use egui::Color32;

pub struct Fluid {

    pub width: usize,
    pub height: usize,

    pub time: f32,
    pub diffusion: f32,
    pub viscosity: f32,

    pub dissipation: f32,

    pub density_r: Vec<f32>,
    pub density_g: Vec<f32>,
    pub density_b: Vec<f32>,

    pub px: Vec<f32>,
    pub py: Vec<f32>,

    pub emitters: Vec<Emitter>,
}

impl Fluid {

    pub fn new(time: f32, diffusion: f32, viscosity: f32) -> Self {

        let width = 100;
        let height = 75;
        let size = width * height;

        Self {

            width,
            height,
            time,
            diffusion,
            viscosity,
            dissipation: 0.995,

            density_r: vec![0.0; size],
            density_g: vec![0.0; size],
            density_b: vec![0.0; size],

            px: vec![0.0; size],
            py: vec![0.0; size],

            //initial emitter values

            emitters: vec![
                {let mut e = Emitter::new(width / 3, height / 2);
                    e.color = Color32::from_rgb(255, 100, 100);
                    e.angle = 0.0_f32.to_radians();
                    e
},
                {let mut e = Emitter::new(2 * width / 3, height / 2);
                    e.color = Color32::from_rgb(100, 100, 255);
                    e.angle = 180.0_f32.to_radians();
                    e
                },
            ],
        }
    }


    pub fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }


    fn inject(&mut self) {
        for emitter in &self.emitters {
            let index_fn = |x, y| x + y * self.width;

            emitter.inject(self.width, self.height, &mut self.density_r, &mut self.density_g, &mut self.density_b, &mut self.px, &mut self.py, index_fn, );
        }
    }


    pub fn step(&mut self) {

        for emitter in &mut self.emitters {
            emitter.update(self.time);
        }

        self.inject();

        // Diffuse velocity (px, py)
        let mut temp_x = self.px.clone();
        let mut temp_y = self.py.clone();
        self.diffuse(&mut temp_x, &self.px, self.viscosity, self.time);
        self.diffuse(&mut temp_y, &self.py, self.viscosity, self.time);
        self.px.copy_from_slice(&temp_x);
        self.py.copy_from_slice(&temp_y);

        self.project();

        // Advect velocity
        let vx0 = self.px.clone();
        let vy0 = self.py.clone();
        let mut new_px = self.px.clone();
        let mut new_py = self.py.clone();
        self.advect(&mut new_px, &vx0, &vx0, &vy0);
        self.advect(&mut new_py, &vy0, &vx0, &vy0);
        self.px.copy_from_slice(&new_px);
        self.py.copy_from_slice(&new_py);

        self.project();

        // Advect density channels
        let r0 = self.density_r.clone();
        let g0 = self.density_g.clone();
        let b0 = self.density_b.clone();

        let mut new_r = self.density_r.clone();
        let mut new_g = self.density_g.clone();
        let mut new_b = self.density_b.clone();

        self.advect(&mut new_r, &r0, &self.px, &self.py);
        self.advect(&mut new_g, &g0, &self.px, &self.py);
        self.advect(&mut new_b, &b0, &self.px, &self.py);

        self.density_r.copy_from_slice(&new_r);
        self.density_g.copy_from_slice(&new_g);
        self.density_b.copy_from_slice(&new_b);

        // Fade
        for d in &mut self.density_r { *d *= self.dissipation; }
        for d in &mut self.density_g { *d *= self.dissipation; }
        for d in &mut self.density_b { *d *= self.dissipation; }
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

                d[idx] = s0*(t0*d0[self.index(i0,j0)] + t1*d0[self.index(i0,j1)]) + s1*(t0*d0[self.index(i1,j0)] + t1*d0[self.index(i1,j1)]);
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
                div[idx] = -0.5 * (self.px[self.index(i+1,j)] - self.px[self.index(i-1,j)] + self.py[self.index(i,j+1)] - self.py[self.index(i,j-1)]) / self.width as f32;
                p[idx] = 0.0;
            }
        }

        // Solve pressure
        for _ in 0..20 {
            for j in 1..self.height-1 {
                for i in 1..self.width-1 {
                    let idx = self.index(i,j);
                    p[idx] = (div[idx] + p[self.index(i-1,j)] + p[self.index(i+1,j)] + p[self.index(i,j-1)] + p[self.index(i,j+1)]) / 4.0;
                }
            }
        }

        // Subtract gradient
        for j in 1..self.height-1 {
            for i in 1..self.width-1 {
                let idx = self.index(i,j);

                self.px[idx] -= 0.5 * (p[self.index(i+1,j)] - p[self.index(i-1,j)]) * self.width as f32;
                self.py[idx] -= 0.5 * (p[self.index(i,j+1)] - p[self.index(i,j-1)]) * self.height as f32;
            }
        }
    }

}