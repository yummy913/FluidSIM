use rand::Rng;
use eframe::egui::Color32;

pub struct Emitter {
    pub x: usize,
    pub y: usize,
    pub strength: f32,
    pub radius: usize,
    pub angle: f32,
    pub rotation_speed: f32,
    pub color: Color32,
}

impl Emitter {

    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            strength: 1.0,
            radius: 1,
            angle: 0.0,
            rotation_speed: 0.0,
            color: Color32::WHITE,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.angle += self.rotation_speed * dt;

        if self.angle > std::f32::consts::TAU {
            self.angle -= std::f32::consts::TAU;
        } else if self.angle < 0.0 {
            self.angle += std::f32::consts::TAU;
        }
    }


    pub fn inject(&self, width: usize, height: usize, density_r: &mut [f32], density_g: &mut [f32], density_b: &mut [f32], px: &mut [f32], py: &mut [f32], index_fn: impl Fn(usize, usize) -> usize, ) {
        
        let mut rng = rand::thread_rng();

        let r_val = self.color.r() as f32 / 255.0 * self.strength;
        let g_val = self.color.g() as f32 / 255.0 * self.strength;
        let b_val = self.color.b() as f32 / 255.0 * self.strength;

        let rad = self.radius as i32;

        for dy in -rad..=rad {
            for dx in -rad..=rad {
                let x = self.x as i32 + dx;
                let y = self.y as i32 + dy;

                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    let idx = index_fn(x as usize, y as usize);

                    // inject colored smoke
                    density_r[idx] += r_val;
                    density_g[idx] += g_val;
                    density_b[idx] += b_val;

                    // velocity spread
                    let spread = (rng.r#gen::<f32>() - 0.5) * 0.5;
                    let angle = self.angle + spread;

                    px[idx] += angle.cos() * self.strength;
                    py[idx] += angle.sin() * self.strength;
                }
            }
        }
    }
}