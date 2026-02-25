use eframe::egui;
use egui::Vec2;
mod fluid;
mod emitter;

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

impl Sim {
    // --- Add this inside impl Sim, not at top level ---
    fn handle_emitter_drag(
        &mut self,
        response: &egui::Response,
        rect: egui::Rect,
        cell_w: f32,
        cell_h: f32,
    ) {
        if !response.dragged() {
            return;
        }

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let grid_x = ((pointer_pos.x - rect.left()) / cell_w) as i32;
            let grid_y = ((pointer_pos.y - rect.top()) / cell_h) as i32;

            if grid_x < 0
                || grid_x >= self.fluid.width as i32
                || grid_y < 0
                || grid_y >= self.fluid.height as i32
            {
                return;
            }

            let mut closest = None;
            let mut closest_dist = f32::MAX;

            for (i, emitter) in self.fluid.emitters.iter().enumerate() {
                let dx = emitter.x as f32 - grid_x as f32;
                let dy = emitter.y as f32 - grid_y as f32;
                let dist = dx * dx + dy * dy;

                if dist < closest_dist {
                    closest_dist = dist;
                    closest = Some(i);
                }
            }

            if let Some(i) = closest {
                self.fluid.emitters[i].x = grid_x as usize;
                self.fluid.emitters[i].y = grid_y as usize;
            }
        }
    }

    fn draw_emitters(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        cell_w: f32,
        cell_h: f32,
    ) {
        for (_i, emitter) in self.fluid.emitters.iter().enumerate() {
            let emitter_x = rect.left() + emitter.x as f32 * cell_w + cell_w * 0.5;
            let emitter_y = rect.top() + emitter.y as f32 * cell_h + cell_h * 0.5;

            let center = egui::pos2(emitter_x, emitter_y);

            painter.circle_filled(
                center,
                8.0,
                emitter.color, // use each emitter's color
            );

            let dir_x = emitter.angle.cos();
            let dir_y = emitter.angle.sin();

            let arrow_length = 20.0;
            let arrow_tip = egui::pos2(
                emitter_x + dir_x * arrow_length,
                emitter_y + dir_y * arrow_length,
            );

            painter.line_segment(
                [center, arrow_tip],
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );
        }
    }
}

impl eframe::App for Sim {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        //
        // fluid controls
        //

        egui::Window::new("Fluid Controls").show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.fluid.time, 0.01..=2.0).text("Timestep"));
            ui.add(egui::Slider::new(&mut self.fluid.viscosity, 0.0..=0.0001).text("Viscosity"));
            ui.add(egui::Slider::new(&mut self.fluid.diffusion, 0.0..=0.01).text("Diffusion"));
            ui.add(egui::Slider::new(&mut self.fluid.dissipation, 0.901..=1.01).text("Dissipation"));
        });

        //
        // emmiter controls
        //

        egui::Window::new("Emitter Controls").show(ctx, |ui| {
            for (i, emitter) in self.fluid.emitters.iter_mut().enumerate() {
                let mut angle_deg = emitter.angle.to_degrees();
                ui.group(|ui| {
                    ui.label(format!("Emitter {}", i + 1));
                    ui.add(egui::Slider::new(&mut emitter.strength, 0.0..=10.0).text("Strength"));
                    ui.add(egui::Slider::new(&mut emitter.rotation_speed, -1.0..=1.0).text("Rotation Speed"));
                    ui.add(egui::Slider::new(&mut emitter.radius, 1..=3).text("Radius"));
                    ui.add(egui::Slider::new(&mut angle_deg, 0.0..=360.0).text("Angle (°)"));
                    ui.color_edit_button_srgba(&mut emitter.color);
                });

                //for angle slider

                if angle_deg >= 360.0 {
                    angle_deg -= 360.0;
                } else if angle_deg < 0.0 {
                    angle_deg += 360.0;
                }
                emitter.angle = angle_deg.to_radians();
            }
        });

        //egui::Window::new("Smoke color").show(ctx, |ui| {
        //    ui.color_edit_button_srgba(&mut self.fluid.color);
        //});

        self.fluid.step();

        //
        // rendering
        //

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
            let rect = response.rect;

            let cell_w = rect.width() / self.fluid.width as f32;
            let cell_h = rect.height() / self.fluid.height as f32;

            for y in 0..self.fluid.height {
                for x in 0..self.fluid.width {
                    let idx = self.fluid.index(x, y);
                
                    let r = self.fluid.density_r[idx].clamp(0.0, 1.0);
                    let g = self.fluid.density_g[idx].clamp(0.0, 1.0);
                    let b = self.fluid.density_b[idx].clamp(0.0, 1.0);
                
                    let color = egui::Color32::from_rgb(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                    );
                
                    let px = rect.left() + x as f32 * cell_w;
                    let py = rect.top() + y as f32 * cell_h;
                
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(px, py), Vec2::new(cell_w, cell_h)),
                        0.0,
                        color,
                    );
                }
            }

            self.handle_emitter_drag(&response, rect, cell_w, cell_h);
            self.draw_emitters(&painter, rect, cell_w, cell_h);

        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2 { x: 1200.0, y: 700.0 }),
        ..Default::default()
    };

    eframe::run_native(
        "Fluid Simulator",
        options,
        Box::new(|_cc| Box::<Sim>::default()),
    )
}