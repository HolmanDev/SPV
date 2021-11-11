#![windows_subsystem = "windows"]
#![allow(unused_assignments)]

use crate::epaint::TextureId;
use eframe::{egui, epi};
use egui::epaint;

fn main() {
    let app = Canvas::default();

    let options = eframe::NativeOptions {
        transparent: true,

        ..Default::default()
    };

    eframe::run_native(Box::new(app), options);
}

fn euler_angle_transformations(
    lotn: f64,
    aop: f64,
    i: f64,
) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
    let x1 = (lotn.to_radians().cos() * aop.to_radians().cos())
        - (lotn.to_radians().sin() * i.to_radians().cos() * aop.to_radians().sin());
    let x2 = (lotn.to_radians().sin() * aop.to_radians().cos())
        + (lotn.to_radians().cos() * i.to_radians().cos() * aop.to_radians().sin());
    let x3 = i.to_radians().sin() * aop.to_radians().sin();

    let y1 = ((0. - lotn.to_radians().cos()) * aop.to_radians().sin())
        - (lotn.to_radians().sin() * i.to_radians().cos() * aop.to_radians().cos());
    let y2 = ((0. - lotn.to_radians().sin()) * aop.to_radians().sin())
        + (lotn.to_radians().cos() * i.to_radians().cos() * aop.to_radians().cos());
    let y3 = i.to_radians().sin() * aop.to_radians().cos();

    let z1 = i.to_radians().sin() * lotn.to_radians().sin();
    let z2 = (0. - i.to_radians().sin()) * lotn.to_radians().cos();
    let z3 = i.to_radians().cos();

    return (x1, x2, x3, y1, y2, y3, z1, z2, z3);
}

fn position(distance: f64, right_ascension: f64, declination: f64) -> (f64, f64, f64) {
    let x =
        distance * (right_ascension.to_radians()).cos() * ((declination + 90.).to_radians()).sin();

    let y =
        distance * (right_ascension.to_radians()).sin() * ((declination + 90.).to_radians()).sin();

    let z = distance * ((declination + 90.).to_radians()).cos();

    return (x, y, z);
}
fn velocity(
    distance: f64,
    right_ascension: f64,
    declination: f64,
    proper_motion_ra: f64,
    proper_motion_dec: f64,
    x: f64,
    y: f64,
    z: f64,
    radial_velocity: f64,
) -> (f64, f64, f64) {
    let proper_motion_x = distance
        * (((right_ascension + ((proper_motion_ra * 0.00027777777777778) / 31556926.))
            .to_radians())
        .cos())
        * ((((declination + ((proper_motion_dec * 0.00027777777777778) / 31556926.)) + 90.)
            .to_radians())
        .sin());

    let proper_motion_y = distance
        * (((right_ascension + ((proper_motion_ra * 0.00027777777777778) / 31556926.))
            .to_radians())
        .sin())
        * ((((declination + ((proper_motion_dec * 0.00027777777777778) / 31556926.)) + 90.)
            .to_radians())
        .sin());

    let proper_motion_z = distance
        * ((((declination + ((proper_motion_dec * 0.00027777777777778) / 31556926.)) + 90.)
            .to_radians())
        .cos());

    let proper_motion_vector_x = proper_motion_x - x;
    let proper_motion_vector_y = proper_motion_y - y;
    let proper_motion_vector_z = proper_motion_z - z;

    let mut normalized_vector_x = 0.;
    let mut normalized_vector_y = 0.;
    let mut normalized_vector_z = 0.;
    if radial_velocity < 0. {
        normalized_vector_x = 0. - (x / (x.powf(2.) + y.powf(2.) + z.powf(2.)).sqrt());
        normalized_vector_y = 0. - (y / (x.powf(2.) + y.powf(2.) + z.powf(2.)).sqrt());
        normalized_vector_z = 0. - (z / (x.powf(2.) + y.powf(2.) + z.powf(2.)).sqrt());
    } else if radial_velocity > 0. {
        normalized_vector_x = x / (x.powf(2.) + y.powf(2.) + z.powf(2.)).sqrt();
        normalized_vector_y = y / (x.powf(2.) + y.powf(2.) + z.powf(2.)).sqrt();
        normalized_vector_z = z / (x.powf(2.) + y.powf(2.) + z.powf(2.)).sqrt();
    } else {
        normalized_vector_x = 0.;
        normalized_vector_y = 0.;
        normalized_vector_z = 0.;
    }
    let radial_velocity_vector_x = normalized_vector_x * radial_velocity;
    let radial_velocity_vector_y = normalized_vector_y * radial_velocity;
    let radial_velocity_vector_z = normalized_vector_z * radial_velocity;

    let x_v = radial_velocity_vector_x + proper_motion_vector_x;
    let y_v = radial_velocity_vector_y + proper_motion_vector_y;
    let z_v = radial_velocity_vector_z + proper_motion_vector_z;

    return (x_v, y_v, z_v);
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]

struct Export {
    x_pos: f64,
    y_pos: f64,
    z_pos: f64,

    x_vel: f64,
    y_vel: f64,
    z_vel: f64,

    new_base_x_x: f64,
    new_base_x_y: f64,
    new_base_x_z: f64,
    new_base_y_x: f64,
    new_base_y_y: f64,
    new_base_y_z: f64,
    new_base_z_x: f64,
    new_base_z_y: f64,
    new_base_z_z: f64,

    name: String,
}

use serde_json;
use std::fs::File;
use std::io::{BufWriter, Write};

fn export_json(
    x: f64,
    y: f64,
    z: f64,
    x_v: f64,
    y_v: f64,
    z_v: f64,
    new_base_x_x: f64,
    new_base_x_y: f64,
    new_base_x_z: f64,
    new_base_y_x: f64,
    new_base_y_y: f64,
    new_base_y_z: f64,
    new_base_z_x: f64,
    new_base_z_y: f64,
    new_base_z_z: f64,
    name_str: String,
) {
    let data = Export {
        x_pos: x,
        y_pos: y,
        z_pos: z,

        x_vel: x_v,
        y_vel: y_v,
        z_vel: z_v,

        new_base_x_x: new_base_x_x,
        new_base_x_y: new_base_x_y,
        new_base_x_z: new_base_x_z,
        new_base_y_x: new_base_y_x,
        new_base_y_y: new_base_y_y,
        new_base_y_z: new_base_y_z,
        new_base_z_x: new_base_z_x,
        new_base_z_y: new_base_z_y,
        new_base_z_z: new_base_z_z,

        name: name_str,
    };

    // write out the file
    let writer = BufWriter::new(File::create("data.json").unwrap());

    serde_json::to_writer_pretty(writer, &data).unwrap();
}

fn export_txt(
    x: f64,
    y: f64,
    z: f64,
    x_v: f64,
    y_v: f64,
    z_v: f64,
    new_base_x_x: f64,
    new_base_x_y: f64,
    new_base_x_z: f64,
    new_base_y_x: f64,
    new_base_y_y: f64,
    new_base_y_z: f64,
    new_base_z_x: f64,
    new_base_z_y: f64,
    new_base_z_z: f64,
    name_str: String,
) {
    let data = Export {
        x_pos: x,
        y_pos: y,
        z_pos: z,

        x_vel: x_v,
        y_vel: y_v,
        z_vel: z_v,

        new_base_x_x: new_base_x_x,
        new_base_x_y: new_base_x_y,
        new_base_x_z: new_base_x_z,
        new_base_y_x: new_base_y_x,
        new_base_y_y: new_base_y_y,
        new_base_y_z: new_base_y_z,
        new_base_z_x: new_base_z_x,
        new_base_z_y: new_base_z_y,
        new_base_z_z: new_base_z_z,

        name: name_str,
    };

    let mut buffer = File::create("data.txt").unwrap();

    buffer
        .write_all(serde_json::to_string(&data).unwrap().as_bytes())
        .unwrap();
}

#[derive(Default)]

pub struct Canvas {
    name_str: String,

    x: f64,
    y: f64,
    z: f64,

    x_v: f64,
    y_v: f64,
    z_v: f64,

    distance: f64,    //In Lightyears
    distance_km: f64, //In km
    distance_str: String,

    declination: f64, //degrees

    declination_degree: f64, //In Degreees
    declination_degree_str: String,

    declination_min: f64, //In Minutes (')
    declination_min_str: String,

    declination_s: f64, //In Seconds ('')
    declination_s_str: String,

    right_ascension: f64, //degrees

    right_ascension_h: f64, //In Hours
    right_ascension_h_str: String,

    right_ascension_min: f64, //In Minutes
    right_ascension_min_str: String,

    right_ascension_s: f64, //In Seconds
    right_ascension_s_str: String,

    radial_velocity: f64, //In km/s
    radial_velocity_str: String,

    proper_motion_ra: f64, //In Arcseconds/year
    proper_motion_ra_str: String,

    proper_motion_dec: f64, //In Arcseconds/year
    proper_motion_dec_str: String,

    lotn: f64,
    aop: f64,
    i: f64,

    lotn_str: String,
    aop_str: String,
    i_str: String,

    new_base_x_x: f64,
    new_base_x_y: f64,
    new_base_x_z: f64,
    new_base_y_x: f64,
    new_base_y_y: f64,
    new_base_y_z: f64,
    new_base_z_x: f64,
    new_base_z_y: f64,
    new_base_z_z: f64,

    general_img: TextureId,
    pos_vel_img: TextureId,
    export_img: TextureId,
    organize_img: TextureId,
    euler_angle_transformations_img: TextureId,

    general_toggle: bool,
    pos_vel_toggle: bool,
    export_toggle: bool,
    euler_angle_transformations_toggle: bool,
}

impl epi::App for Canvas {
    fn name(&self) -> &str {
        "SPV"
    }

    #[allow(unused_variables)]

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        #[cfg(feature = "persistence")]
        if let Some(storage) = storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        let mut style: egui::Style = (*ctx.style()).clone();

        style.visuals.extreme_bg_color = egui::Color32::from_rgb(255, 255, 250);

        style.visuals.faint_bg_color = egui::Color32::from_rgb(178, 176, 155);

        style.visuals.code_bg_color = egui::Color32::from_rgb(235, 231, 233);

        style.visuals.hyperlink_color = egui::Color32::from_rgb(255, 69, 0);

        style.visuals.override_text_color = Some(egui::Color32::from_rgb(32, 30, 31));

        style.visuals.window_corner_radius = 0.1;

        style.visuals.button_frame = true;

        style.visuals.collapsing_header_frame = true;

        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(235, 231, 233);

        style.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;

        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(255, 69, 0);

        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(255, 0, 0);

        style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(255, 0, 0);

        ctx.set_style(style);

        let image_data_general = include_bytes!("data/MenuGeneral.png");
        let image_general =
            image::load_from_memory(image_data_general).expect("Failed to load image");
        let image_buffer_general = image_general.to_rgba8();
        let size_general = (350 as usize, 100 as usize);
        let pixels_general = image_buffer_general.into_vec();
        let pixels_general: Vec<_> = pixels_general
            .chunks_exact(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        // Allocate a texture:
        self.general_img = frame
            .tex_allocator()
            .alloc_srgba_premultiplied(size_general, &pixels_general);

        let image_data_pos_vel = include_bytes!("data/MenuPos&Vel.png");
        let image_pos_vel =
            image::load_from_memory(image_data_pos_vel).expect("Failed to load image");
        let image_buffer_pos_vel = image_pos_vel.to_rgba8();
        let size_pos_vel = (350 as usize, 100 as usize);
        let pixels_pos_vel = image_buffer_pos_vel.into_vec();
        let pixels_pos_vel: Vec<_> = pixels_pos_vel
            .chunks_exact(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        // Allocate a texture:
        self.pos_vel_img = frame
            .tex_allocator()
            .alloc_srgba_premultiplied(size_pos_vel, &pixels_pos_vel);

        let image_data_export = include_bytes!("data/MenuExport.png");
        let image_export =
            image::load_from_memory(image_data_export).expect("Failed to load image");
        let image_buffer_export = image_export.to_rgba8();
        let size_export = (350 as usize, 100 as usize);
        let pixels_export = image_buffer_export.into_vec();
        let pixels_export: Vec<_> = pixels_export
            .chunks_exact(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        // Allocate a texture:
        self.export_img = frame
            .tex_allocator()
            .alloc_srgba_premultiplied(size_export, &pixels_export);

        let image_data_organize = include_bytes!("data/MenuOrganize.png");
        let image_organize =
            image::load_from_memory(image_data_organize).expect("Failed to load image");
        let image_buffer_organize = image_organize.to_rgba8();
        let size_organize = (350 as usize, 100 as usize);
        let pixels_organize = image_buffer_organize.into_vec();
        let pixels_organize: Vec<_> = pixels_organize
            .chunks_exact(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        // Allocate a texture:
        self.organize_img = frame
            .tex_allocator()
            .alloc_srgba_premultiplied(size_organize, &pixels_organize);

        let image_data_euler_angle_transformations =
            include_bytes!("data/MenuEulerAngleTransformations.png");
        let image_euler_angle_transformations =
            image::load_from_memory(image_data_euler_angle_transformations)
                .expect("Failed to load image");
        let image_buffer_euler_angle_transformations = image_euler_angle_transformations.to_rgba8();
        let size_euler_angle_transformations = (350 as usize, 100 as usize);
        let pixels_euler_angle_transformations =
            image_buffer_euler_angle_transformations.into_vec();
        let pixels_euler_angle_transformations: Vec<_> = pixels_euler_angle_transformations
            .chunks_exact(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        // Allocate a texture:
        self.euler_angle_transformations_img = frame.tex_allocator().alloc_srgba_premultiplied(
            size_euler_angle_transformations,
            &pixels_euler_angle_transformations,
        );
    }

    #[cfg(feature = "persistence")]

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn clear_color(&self) -> egui::Rgba {
        egui::Rgba::TRANSPARENT
    }

    fn warm_up_enabled(&self) -> bool {
        return true;
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::SidePanel::left("Tabs").show(ctx, |ui| {
            if ui
                .add(egui::ImageButton::new(
                    self.general_img,
                    egui::Vec2::new(140., 40.),
                ))
                .clicked()
            {
                self.general_toggle = !self.general_toggle
            }

            if ui
                .add(egui::ImageButton::new(
                    self.pos_vel_img,
                    egui::Vec2::new(140., 40.),
                ))
                .clicked()
            {
                self.pos_vel_toggle = !self.pos_vel_toggle
            }

            if ui
                .add(egui::ImageButton::new(
                    self.euler_angle_transformations_img,
                    egui::Vec2::new(140., 40.),
                ))
                .clicked()
            {
                self.euler_angle_transformations_toggle = !self.euler_angle_transformations_toggle
            }

            ui.separator();
            if ui
                .add(egui::ImageButton::new(
                    self.export_img,
                    egui::Vec2::new(140., 40.),
                ))
                .clicked()
            {
                self.export_toggle = !self.export_toggle
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.organize_img,
                        egui::Vec2::new(140., 40.),
                    ))
                    .clicked()
                {
                    ui.ctx().memory().reset_areas();
                }
                ui.separator();
            });
        });
        egui::TopBottomPanel::bottom("Result").show(ctx, |ui| {
            ui.add(egui::Label::new(format!("Resulting position (km)")).heading());

            ui.add(
                egui::Label::new(format!(
                    "{:?}",
                    position(
                        self.distance_km.clone(),
                        self.right_ascension.clone(),
                        self.declination.clone()
                    )
                ))
                .monospace(),
            );

            ui.separator();

            ui.add(egui::Label::new(format!("Resulting velocity (km/s)")).heading());

            ui.add(
                egui::Label::new(format!(
                    "{:?}",
                    velocity(
                        self.distance_km.clone(),
                        self.right_ascension.clone(),
                        self.declination.clone(),
                        self.proper_motion_ra.clone(),
                        self.proper_motion_dec.clone(),
                        self.x.clone(),
                        self.y.clone(),
                        self.z.clone(),
                        self.radial_velocity.clone(),
                    )
                ))
                .monospace(),
            );

            ui.separator();

            ui.add(egui::Label::new(format!("New base")).heading());

            ui.add(
                egui::Label::new(format!(
                    "X-NEW: x({:?}), y({:?}), z({:?})",
                    euler_angle_transformations(self.lotn, self.aop, self.i,).0,
                    euler_angle_transformations(self.lotn, self.aop, self.i,).1,
                    euler_angle_transformations(self.lotn, self.aop, self.i,).2
                ))
                .monospace(),
            );
            ui.add(
                egui::Label::new(format!(
                    "Y-NEW: x({:?}), y({:?}), z({:?})",
                    euler_angle_transformations(self.lotn, self.aop, self.i,).3,
                    euler_angle_transformations(self.lotn, self.aop, self.i,).4,
                    euler_angle_transformations(self.lotn, self.aop, self.i,).5
                ))
                .monospace(),
            );
            ui.add(
                egui::Label::new(format!(
                    "Z-NEW: x({:?}), y({:?}), z({:?})",
                    euler_angle_transformations(self.lotn, self.aop, self.i,).6,
                    euler_angle_transformations(self.lotn, self.aop, self.i,).7,
                    euler_angle_transformations(self.lotn, self.aop, self.i,).8
                ))
                .monospace(),
            );

            self.x = position(
                self.distance_km.clone(),
                self.right_ascension.clone(),
                self.declination.clone(),
            )
            .0;

            self.y = position(
                self.distance_km.clone(),
                self.right_ascension.clone(),
                self.declination.clone(),
            )
            .1;

            self.z = position(
                self.distance_km.clone(),
                self.right_ascension.clone(),
                self.declination.clone(),
            )
            .2;

            self.x_v = velocity(
                self.distance_km.clone(),
                self.right_ascension.clone(),
                self.declination.clone(),
                self.proper_motion_ra.clone(),
                self.proper_motion_dec.clone(),
                self.x.clone(),
                self.y.clone(),
                self.z.clone(),
                self.radial_velocity.clone(),
            )
            .0;

            self.y_v = velocity(
                self.distance_km.clone(),
                self.right_ascension.clone(),
                self.declination.clone(),
                self.proper_motion_ra.clone(),
                self.proper_motion_dec.clone(),
                self.x.clone(),
                self.y.clone(),
                self.z.clone(),
                self.radial_velocity.clone(),
            )
            .1;

            self.z_v = velocity(
                self.distance_km.clone(),
                self.right_ascension.clone(),
                self.declination.clone(),
                self.proper_motion_ra.clone(),
                self.proper_motion_dec.clone(),
                self.x.clone(),
                self.y.clone(),
                self.z.clone(),
                self.radial_velocity.clone(),
            )
            .2;

            self.new_base_x_x = euler_angle_transformations(self.lotn, self.aop, self.i).0;
            self.new_base_x_y = euler_angle_transformations(self.lotn, self.aop, self.i).1;
            self.new_base_x_z = euler_angle_transformations(self.lotn, self.aop, self.i).2;
            self.new_base_y_x = euler_angle_transformations(self.lotn, self.aop, self.i).3;
            self.new_base_y_y = euler_angle_transformations(self.lotn, self.aop, self.i).4;
            self.new_base_y_z = euler_angle_transformations(self.lotn, self.aop, self.i).5;
            self.new_base_z_x = euler_angle_transformations(self.lotn, self.aop, self.i).6;
            self.new_base_z_y = euler_angle_transformations(self.lotn, self.aop, self.i).7;
            self.new_base_z_z = euler_angle_transformations(self.lotn, self.aop, self.i).8;
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let naming_window = egui::Window::new("Name")
                .auto_sized()
                .collapsible(true)
                .resizable(false);

            if self.general_toggle == true {
                ui.vertical(|_ui| {
                    naming_window.show(ctx, |ui| {
                        ui.add(egui::Label::new(format!("System name")).heading());

                        let response = ui.add(egui::TextEdit::singleline(&mut self.name_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {}

                        ui.add(egui::Label::new(format!("{}", self.name_str)).monospace());
                    });
                });
            }

            let distance_window = egui::Window::new("Distance")
                .auto_sized()
                .collapsible(true)
                .resizable(false);

            if self.pos_vel_toggle == true {
                ui.vertical(|_ui| {
                    distance_window.show(ctx, |ui| {
                        ui.add(egui::Label::new(format!("Distance (lightyears)")).heading());

                        let response = ui.add(egui::TextEdit::singleline(&mut self.distance_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.distance = self.distance_str.clone().parse().unwrap();
                        }

                        ui.add(egui::Label::new(format!("{} ly", self.distance)).monospace());

                        self.distance_km = self.distance * 9.461 * 10_f64.powf(12.);

                        ui.add(egui::Label::new(format!("{} km", self.distance_km)).monospace());
                    });
                });
            }

            let ra_window = egui::Window::new("Right ascension")
                .auto_sized()
                .collapsible(true)
                .resizable(false);

            if self.pos_vel_toggle == true {
                ui.vertical(|_ui| {
                    ra_window.show(ctx, |ui| {
                        ui.add(egui::Label::new(format!("Right ascension")).heading());

                        ui.add(egui::Label::new(format!("Hours (h)")).monospace());

                        let response =
                            ui.add(egui::TextEdit::singleline(&mut self.right_ascension_h_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.right_ascension_h =
                                self.right_ascension_h_str.clone().parse().unwrap();
                        }

                        ui.add(
                            egui::Label::new(format!("{}h", self.right_ascension_h)).monospace(),
                        );

                        ui.add(egui::Label::new(format!("Minutes (m)")).monospace());

                        let response = ui.add(egui::TextEdit::singleline(
                            &mut self.right_ascension_min_str,
                        ));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.right_ascension_min =
                                self.right_ascension_min_str.clone().parse().unwrap();
                        }

                        ui.add(
                            egui::Label::new(format!("{}m", self.right_ascension_min)).monospace(),
                        );

                        ui.add(egui::Label::new(format!("Seconds (s)")).monospace());

                        let response =
                            ui.add(egui::TextEdit::singleline(&mut self.right_ascension_s_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.right_ascension_s =
                                self.right_ascension_s_str.clone().parse().unwrap();
                        }

                        ui.add(
                            egui::Label::new(format!("{}s", self.right_ascension_s)).monospace(),
                        );

                        self.right_ascension = (self.right_ascension_h * 15.)
                            + (self.right_ascension_min * (1. / 4.))
                            + (self.right_ascension_s * (1. / 240.));

                        ui.add(egui::Label::new(format!("Total")).heading());

                        ui.add(egui::Label::new(format!("{}°", self.right_ascension)).monospace());
                    });
                });
            }

            let dec_window = egui::Window::new("Declination")
                .auto_sized()
                .collapsible(true)
                .resizable(false);

            if self.pos_vel_toggle == true {
                ui.vertical(|_ui| {
                    dec_window.show(ctx, |ui| {
                        ui.add(egui::Label::new(format!("Declination")).heading());

                        ui.add(egui::Label::new(format!("Degrees (°)")).monospace());

                        let response =
                            ui.add(egui::TextEdit::singleline(&mut self.declination_degree_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.declination_degree =
                                self.declination_degree_str.clone().parse().unwrap();
                        }

                        ui.add(
                            egui::Label::new(format!("{}°", self.declination_degree)).monospace(),
                        );

                        ui.add(egui::Label::new(format!("Minutes (')")).monospace());

                        let response =
                            ui.add(egui::TextEdit::singleline(&mut self.declination_min_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.declination_min =
                                self.declination_min_str.clone().parse().unwrap();
                        }

                        ui.add(egui::Label::new(format!("{}'", self.declination_min)).monospace());

                        ui.add(egui::Label::new(format!("Seconds ('')")).monospace());

                        let response =
                            ui.add(egui::TextEdit::singleline(&mut self.declination_s_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.declination_s = self.declination_s_str.clone().parse().unwrap();
                        }

                        ui.add(egui::Label::new(format!("{}''", self.declination_s)).monospace());

                        self.declination = self.declination_degree
                            + (self.declination_min / 60.)
                            + (self.declination_s / 3600.);

                        ui.add(egui::Label::new(format!("Total")).heading());

                        ui.add(egui::Label::new(format!("{}°", self.declination)).monospace());
                    });
                });
            }

            let rv_window = egui::Window::new("Radial velocity")
                .auto_sized()
                .collapsible(true)
                .resizable(false);

            if self.pos_vel_toggle == true {
                ui.vertical(|_ui| {
                    rv_window.show(ctx, |ui| {
                        ui.add(egui::Label::new(format!("Radial velocity (km/s)")).heading());

                        let response =
                            ui.add(egui::TextEdit::singleline(&mut self.radial_velocity_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.radial_velocity =
                                self.radial_velocity_str.clone().parse().unwrap();
                        }

                        ui.add(
                            egui::Label::new(format!("{} km/s", self.radial_velocity)).monospace(),
                        );
                    });
                });
            }

            let pm_window = egui::Window::new("Proper motion")
                .auto_sized()
                .collapsible(true)
                .resizable(false);

            if self.pos_vel_toggle == true {
                pm_window.show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.add(egui::Label::new(format!("Proper motion")).heading());

                        ui.add(
                            egui::Label::new(format!("Right ascension (arcsecons/year)"))
                                .monospace(),
                        );

                        let response =
                            ui.add(egui::TextEdit::singleline(&mut self.proper_motion_ra_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.proper_motion_ra =
                                self.proper_motion_ra_str.clone().parse().unwrap();
                        }

                        ui.add(
                            egui::Label::new(format!("{} as/yr", self.proper_motion_ra))
                                .monospace(),
                        );

                        ui.add(
                            egui::Label::new(format!("Declination (arcsecons/year)")).monospace(),
                        );

                        let response =
                            ui.add(egui::TextEdit::singleline(&mut self.proper_motion_dec_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.proper_motion_dec =
                                self.proper_motion_dec_str.clone().parse().unwrap();
                        }

                        ui.add(
                            egui::Label::new(format!("{} as/yr", self.proper_motion_dec))
                                .monospace(),
                        );
                    });
                });
            }

            let euler_angle_transformations_window =
                egui::Window::new("Euler angle transformations")
                    .auto_sized()
                    .collapsible(true)
                    .resizable(false);

            if self.euler_angle_transformations_toggle == true {
                euler_angle_transformations_window.show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.add(egui::Label::new(format!("Angle values")).heading());

                        ui.add(egui::Label::new(format!("Longitude of the node (Ω)")).monospace());

                        let response = ui.add(egui::TextEdit::singleline(&mut self.lotn_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.lotn = self.lotn_str.clone().parse().unwrap();
                        }

                        ui.add(egui::Label::new(format!("{} degrees", self.lotn)).monospace());

                        ui.add(egui::Label::new(format!("Argument of periastron (ω)")).monospace());

                        let response = ui.add(egui::TextEdit::singleline(&mut self.aop_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.aop = self.aop_str.clone().parse().unwrap();
                        }

                        ui.add(egui::Label::new(format!("{} degrees", self.aop)).monospace());

                        ui.add(egui::Label::new(format!("Inclination (i)")).monospace());

                        let response = ui.add(egui::TextEdit::singleline(&mut self.i_str));

                        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            self.i = self.i_str.clone().parse().unwrap();
                        }

                        ui.add(egui::Label::new(format!("{} degrees", self.i)).monospace());
                    });
                });
            }

            let export_window = egui::Window::new("Export file")
                .auto_sized()
                .collapsible(true)
                .resizable(false);

            if self.export_toggle == true {
                export_window.show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.add(egui::Label::new(format!("Export file")).heading());

                        ui.horizontal_wrapped(|ui| {
                            if ui.add(egui::Button::new("JSON")).clicked() {
                                export_json(
                                    self.x,
                                    self.y,
                                    self.z,
                                    self.x_v,
                                    self.y_v,
                                    self.z_v,
                                    self.new_base_x_x,
                                    self.new_base_x_y,
                                    self.new_base_x_z,
                                    self.new_base_y_x,
                                    self.new_base_y_y,
                                    self.new_base_y_z,
                                    self.new_base_z_x,
                                    self.new_base_z_y,
                                    self.new_base_z_z,
                                    self.name_str.clone(),
                                );
                            }

                            if ui.add(egui::Button::new("TXT")).clicked() {
                                export_txt(
                                    self.x,
                                    self.y,
                                    self.z,
                                    self.x_v,
                                    self.y_v,
                                    self.z_v,
                                    self.new_base_x_x,
                                    self.new_base_x_y,
                                    self.new_base_x_z,
                                    self.new_base_y_x,
                                    self.new_base_y_y,
                                    self.new_base_y_z,
                                    self.new_base_z_x,
                                    self.new_base_z_y,
                                    self.new_base_z_z,
                                    self.name_str.clone(),
                                );
                            }
                        });
                    });
                });
            }
        });
    }
}
