use eframe::egui;
mod fluid;

use fluid::Fluid;

struct Sim {
    fluid: Fluid,
}

impl Default for Sim {
    fn default() -> Self {
        Self {
            fluid: Fluid::new(0.5, 0.0, 0.0),
        }
    }
}

impl eframe::App for Sim {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

                egui::Window::new("Fluid Controls").show(ctx, |ui| {
            ui.add(
                egui::Slider::new(&mut self.fluid.time, 0.01..=2.0)
                    .text("Timestep")
            );
        
            ui.add(
                egui::Slider::new(&mut self.fluid.viscosity, 0.0..=0.01)
                    .text("Viscosity")
            );
        
            ui.add(
                egui::Slider::new(&mut self.fluid.diffusion, 0.0..=0.01)
                    .text("Diffusion")
            );
        
            ui.add(
                egui::Slider::new(&mut self.fluid.injection_strength, 0.0..=10.0)
                    .text("Injection Strength")
            );
        
            ui.add(
                egui::Slider::new(&mut self.fluid.injection_radius, 1..=10)
                    .text("Injection Radius")
            );
        
            ui.add(
                egui::Slider::new(&mut self.fluid.dissipation, 0.90..=1.0)
                    .text("Dissipation")
            );
        });

        self.fluid.step();

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click());
            let rect = response.rect;

            let cell_w = rect.width() / self.fluid.width as f32;
            let cell_h = rect.height() / self.fluid.height as f32;

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let x = ((pos.x - rect.left()) / cell_w) as usize;
                    let y = ((pos.y - rect.top()) / cell_h) as usize;

                    self.fluid.push_velocity(x, y, 50.0, 20);
                }
            }

            // Draw density
            for y in 0..self.fluid.height {
                for x in 0..self.fluid.width {
                    let idx = x + y*self.fluid.width;
                    let d = self.fluid.density[idx];

                    let color = egui::Color32::from_gray(d.min(255.0) as u8);

                    let px = rect.left() + x as f32 * cell_w;
                    let py = rect.top() + y as f32 * cell_h;

                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(px, py),
                            egui::vec2(cell_w, cell_h),
                        ),
                        0.0,
                        color,
                    );
                }
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Fluid Continuous Emitter",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::<Sim>::default()),
    )
}