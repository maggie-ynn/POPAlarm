use crate::egui::Color32;
use crate::egui::Pos2;
use egui_extras::image::RetainedImage;
use eframe::egui;
use tray_icon::{
    menu::{MenuEvent},
    TrayEvent,
};

use chrono::{DateTime, Timelike, Local, Utc, FixedOffset, TimeZone};
use ini::Ini;

use std::fs;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};

pub struct POPAlarm {
    config_path: String,
    quit_index: u32,
    settings_index: u32,
    time: f32,
    time2show: String,
    tikpop: bool,
    visible: bool,
    last_pos_x: f32,
    last_pos_y: f32,
    last_visible: bool,
    sound_path: String,
    countdown: String,
    countdown_index: u32,
    inited: bool,
    countdown_start: bool,
    countdown_start_time: i64,
    in_time_popup: bool,
    pos_pc: i32,
    pos_dir: String,
    init_x: f32,
    init_y: f32,
    custom_bg_color: String,
    custom_border_color: String,
    custom_number_bg_color: String,
    custom_number_color: String,
    custom_clock_bg_color: String,
    tips_store: String,
    show_tips: String,
    font_path: String,
    show_time: f32,
    now: DateTime<FixedOffset>,
    time_countdown_target: u32,
    image: Result<RetainedImage, String>,
    init_show: i32,
    timezone: i32,
    custom_timezone: bool,
    time_font: String,
    round: bool,
    time_countdown: bool,
    show_settings: bool,
    settings_status: String,
    show_time_input: String,
    pos_pc_input: String,
    timezone_input: String
}

impl POPAlarm {
    pub fn new(
        config_path: String,
        quit_index: u32,
        settings_index: u32,
        time2show: String,
        sound_path: String,
        countdown: String,
        countdown_index: u32,
        pos_dir: String,
        pos_pc: i32,
        custom_bg_color: String,
        custom_border_color: String,
        custom_number_bg_color: String,
        custom_number_color: String,
        custom_clock_bg_color: String,
        tips_store: String,
        font_path: String,
        show_time: f32,
        image: Result<RetainedImage, String>,
        init_show: i32,
        timezone: i32,
        custom_timezone: bool,
        time_font: String,
        round: bool,
        time_countdown: bool
    ) -> Result<POPAlarm, &'static str> {
        Ok(POPAlarm {
            config_path,
            quit_index,
            settings_index,
            time: 0.0,
            time2show,
            tikpop: false,
            visible: true,
            last_pos_x: 0.0,
            last_pos_y: 0.0,
            last_visible: false,
            sound_path,
            countdown,
            countdown_index,
            inited: false,
            countdown_start: false,
            countdown_start_time: 0,
            in_time_popup: false,
            pos_pc,
            pos_dir,
            init_x: 0.0,
            init_y: 0.0,
            custom_bg_color,
            custom_border_color,
            custom_number_bg_color,
            custom_number_color,
            custom_clock_bg_color,
            tips_store,
            show_tips: "".to_string(),
            font_path,
            show_time,
            now: Local::now().into(),
            time_countdown_target: 0,
            image,
            init_show,
            timezone,
            custom_timezone,
            time_font,
            round,
            time_countdown,
            show_settings: false,
            settings_status: "".to_string(),
            show_time_input: if show_time == 0.0 { "".to_string() } else { show_time.to_string() },
            pos_pc_input: if pos_pc == -1 { "".to_string() } else { pos_pc.to_string() },
            timezone_input: if custom_timezone { timezone.to_string() } else { "".to_string() }
        })
    }

    fn save_settings(&mut self) -> Result<(), String> {
        let mut show_time_raw = 0.0;
        if self.show_time_input.trim() != "" {
            show_time_raw = self.show_time_input.trim().parse::<f32>().map_err(|_| "Show time must be a number".to_string())?;
        }

        let mut pos_pc = -1;
        if self.pos_pc_input.trim() != "" {
            pos_pc = self.pos_pc_input.trim().parse::<i32>().map_err(|_| "Position percent must be an integer".to_string())?;
        }

        let mut timezone = 0;
        let mut custom_timezone = false;
        if self.timezone_input.trim() != "" {
            timezone = self.timezone_input.trim().parse::<i32>().map_err(|_| "Timezone must be an integer hour offset".to_string())?;
            custom_timezone = true;
        }

        self.pos_pc = pos_pc;
        self.show_time = if show_time_raw == 0.0 { 100.0 } else { show_time_raw / 16.0 };
        self.timezone = timezone;
        self.custom_timezone = custom_timezone;

        let mut ini = Ini::new();
        ini.with_section(Some("Config")).set("time", self.time2show.clone());
        if self.countdown.trim() != "" { ini.with_section(Some("Config")).set("countdown", self.countdown.clone()); }
        if self.pos_pc == -1 {
            ini.with_section(Some("Config")).set("pos", self.pos_dir.clone());
        } else {
            ini.with_section(Some("Config")).set("pos", format!("{},{}%", self.pos_dir, self.pos_pc));
        }
        if !self.round { ini.with_section(Some("Config")).set("round", "0"); }
        if show_time_raw != 0.0 { ini.with_section(Some("Config")).set("show_time", show_time_raw.to_string()); }
        if custom_timezone { ini.with_section(Some("Config")).set("timezone", self.timezone.to_string()); }
        if self.tips_store.trim() != "" { ini.with_section(Some("Config")).set("tips", self.tips_store.clone()); }
        if self.font_path.trim() != "" { ini.with_section(Some("Config")).set("font_path", self.font_path.clone()); }
        if self.time_font.trim() != "" { ini.with_section(Some("Config")).set("time_font", self.time_font.clone()); }
        if self.custom_bg_color.trim() != "" { ini.with_section(Some("Config")).set("bg_color", self.custom_bg_color.clone()); }
        if self.custom_border_color.trim() != "" { ini.with_section(Some("Config")).set("border_color", self.custom_border_color.clone()); }
        if self.custom_number_bg_color.trim() != "" { ini.with_section(Some("Config")).set("number_bg_color", self.custom_number_bg_color.clone()); }
        if self.custom_number_color.trim() != "" { ini.with_section(Some("Config")).set("number_color", self.custom_number_color.clone()); }
        if self.custom_clock_bg_color.trim() != "" { ini.with_section(Some("Config")).set("clock_bg_color", self.custom_clock_bg_color.clone()); }
        if self.time_countdown { ini.with_section(Some("Config")).set("time_countdown", "1"); }
        ini.write_to_file(&self.config_path).map_err(|e| e.to_string())?;
        Ok(())
    }
}


impl eframe::App for POPAlarm {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.inited == false {
            self.inited = true;
            let mut fonts = egui::FontDefinitions::default();
            if self.font_path == "" {
                #[cfg(target_os = "windows")]
                {
                    self.font_path = "C:/Windows/Fonts/msyh.ttc".to_string();
                }
                #[cfg(target_os = "macos")]
                {
                    self.font_path = "/System/Library/Fonts/STHeiti Light.ttc".to_string();
                }
            }
            let result = std::fs::read(&self.font_path);
            if let Ok(font) = result {
                fonts.font_data.insert(
                    "other_font".to_owned(),
                    egui::FontData::from_owned(font)
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "other_font".to_owned());
            }

            if self.time_font != "" {
                let result = std::fs::read(&self.time_font);
                if let Ok(font) = result {
                    fonts.font_data.insert(
                        "time_font".to_owned(),
                        egui::FontData::from_owned(font)
                    );
                } else {
                    self.time_font = "".to_string();
                }
            } else {
                fonts.font_data.insert(
                    "time_font".to_owned(),
                    egui::FontData::from_static(include_bytes!("../assets/font.ttf")),
                );
            }
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "time_font".to_owned());
            ctx.set_fonts(fonts);

            if self.init_show == 0 {
                self.visible = false;
                frame.set_visible(false);
            }

            if self.show_time == 0.0 {
                self.show_time = 100.0;
            } else {
                self.show_time = self.show_time / 16.0;
            }
            if self.time_countdown == true && self.time2show != "" {
                let time2show_arr: Vec<&str> = self.time2show.split(',').collect();
                for x in &time2show_arr {
                    let single_time: Vec<&str> = x.split(':').collect();
                    if single_time[0] != "" && single_time[1] != "" && single_time[2] != "" {
                        self.time_countdown_target = single_time[0].parse::<u32>().unwrap() * 3600 + single_time[1].parse::<u32>().unwrap() * 60 + single_time[2].parse::<u32>().unwrap();
                        break;
                    }
                }
            }
        }
        
        let mut begin_tik = |index, in_time_popup| {
            self.last_visible = self.visible;
            if self.last_visible == true {
                if let Some(pos) = frame.get_window_pos() {
                    self.last_pos_x = pos.x;
                    self.last_pos_y = pos.y;
                }
            }
            self.visible = true;
            frame.set_visible(self.visible);
            self.time = 0.0;

            self.init_y = 50.0;
            self.init_x = -320.0;
            if self.pos_pc != -1 {
                if let Some(egui::Vec2 { x, y }) = frame.info().window_info.monitor_size {
                    let pos = self.pos_pc as f32 / 100.0 * y;
                    self.init_y = pos;
                    if self.pos_dir == "right" {
                        self.init_x = x as f32;
                    }
                }
            }

            if let Some(egui::Vec2 { x, y }) = frame.info().window_info.monitor_position {
                self.init_x = x as f32 + &self.init_x;
                self.init_y = y as f32 + &self.init_y;
            }
            frame.set_window_pos(Pos2::new(self.init_x, self.init_y));
            if self.sound_path != "" {
                let mut path = "".to_string();
                let sound_path_arr: Vec<&str> = self.sound_path.split('*').collect();
                let normal_sound:Vec<&str> = sound_path_arr[1].split('|').collect();
                if sound_path_arr.len() == 2 || in_time_popup == true {
                    if normal_sound.len() > index {
                        path = sound_path_arr[0].to_owned() + &normal_sound[index];
                    } else {
                        path = sound_path_arr[0].to_owned() + &normal_sound[0];
                    }
                } else if sound_path_arr.len() == 3 {
                    let countdown_sound:Vec<&str> = sound_path_arr[2].split('|').collect();
                    if countdown_sound.len() > index {
                        path = sound_path_arr[0].to_owned() + &countdown_sound[index];
                    } else {
                        path = sound_path_arr[0].to_owned() + &countdown_sound[0];
                    }
                }
                if path != "" {
                    let result = fs::File::open(&path);
                    if let Ok(file) = result {
                        let file = BufReader::new(file);
                        std::thread::spawn(move || {
                            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                            let source = Decoder::new(file).unwrap();
                            let sink = Sink::try_new(&stream_handle).unwrap();
                            sink.append(source);
                            sink.sleep_until_end();
                        });
                    }
                }
            }
            if self.tips_store != "" {
                let mut tips = "".to_string();
                let tips_arr: Vec<&str> = self.tips_store.split('*').collect();
                let normal_tips:Vec<&str> = tips_arr[0].split('|').collect();
                if tips_arr.len() == 1 || in_time_popup == true {
                    if normal_tips.len() > index {
                        tips = normal_tips[index].to_string();
                    } else {
                        tips = normal_tips[0].to_string();
                    }
                } else if tips_arr.len() == 2 {
                    let countdown_tips:Vec<&str> = tips_arr[1].split('|').collect();
                    if countdown_tips.len() > index {
                        tips = countdown_tips[index].to_string();
                    } else {
                        tips = countdown_tips[0].to_string();
                    }
                }
                if tips != "" {
                    self.show_tips = tips;
                }
            }
            ctx.request_repaint();
        };

        if self.custom_timezone == true {
            let tz_offset;
            if self.timezone < 0 {
                tz_offset = FixedOffset::west_opt(self.timezone * 3600).unwrap();
            } else {
                tz_offset = FixedOffset::east_opt(self.timezone * 3600).unwrap();
            }
            let timezone: FixedOffset = TimeZone::from_offset(&tz_offset);
            self.now = Utc::now().with_timezone(&timezone);
        } else {
            self.now = Local::now().into();
            self.now = self.now.with_hour(Local::now().hour()).unwrap();
        }
        let mut custom_clock = "".to_string();
        if self.countdown_start == true && self.in_time_popup == false {
            let timestamp = self.now.timestamp();
            let over_time = (timestamp - self.countdown_start_time) as i32;
            if self.countdown == "" {
                if over_time > 600 {
                    self.countdown_start_time = timestamp;
                    if self.tikpop == false {
                        begin_tik(0, self.in_time_popup);
                        self.tikpop = true;
                    }
                }
                let left_time = 600.0 - over_time as f32;
                let minute = (left_time / 60.0) as u32;
                let second = (left_time % 60.0) as u32;
                custom_clock = format!("00:{:02}:{:02}", minute, second);
            } else {
                let countdown_arr: Vec<&str> = self.countdown.split(',').collect();
                let mut total_time:i32 = 0;
                let mut first_time:i32 = 0;
                let mut index:i32 = 0;
                for x in &countdown_arr {
                    let single_time: Vec<&str> = x.split(':').collect();
                    let mut cur_time:i32 = 0;
                    if single_time[0] != "" {
                        cur_time = cur_time + single_time[0].to_string().parse::<i32>().unwrap() * 3600;
                    }
                    if single_time[1] != "" {
                        cur_time = cur_time + single_time[1].to_string().parse::<i32>().unwrap() * 60;
                    }
                    if single_time[2] != "" {
                        cur_time = cur_time + single_time[2].to_string().parse::<i32>().unwrap();
                    }
                    if first_time == 0 {
                        first_time = cur_time;
                    }
                    total_time = total_time + cur_time;
                    if self.tikpop == false && over_time == total_time.into() {
                        begin_tik(index.try_into().unwrap(), self.in_time_popup);
                        self.tikpop = true;
                    } else if over_time < total_time {
                        let left_time = (total_time - over_time) as f32;
                        let hour = (left_time / 60.0 / 60.0) as u32;
                        let minute = (left_time / 60.0) as u32;
                        let second = (left_time % 60.0) as u32;
                        custom_clock = format!("{:02}:{:02}:{:02}", hour, minute, second);
                        break;
                    }
                    index = index + 1;
                }
                if custom_clock == "" {
                    self.countdown_start_time = timestamp;
                    let left_time = first_time as f32;
                    let hour = (left_time / 60.0 / 60.0) as u32;
                    let minute = (left_time / 60.0) as u32;
                    let second = (left_time % 60.0) as u32;
                    custom_clock = format!("{:02}:{:02}:{:02}", hour, minute, second);
                    if self.tikpop == false {
                        begin_tik(0, self.in_time_popup);
                        self.tikpop = true;
                    }
                }
            }
        }
        if self.countdown_start == false && self.time_countdown == true && self.time_countdown_target != 0 {
            let hour = self.now.hour();
            let minute = self.now.minute();
            let second = self.now.second();
            let mut gap_time = hour * 3600 + minute * 60 + second;
            if gap_time > self.time_countdown_target {
                gap_time = self.time_countdown_target + 24 * 3600 - gap_time;
            } else {
                gap_time = self.time_countdown_target - gap_time;
            }
            custom_clock = format!("{:02}:{:02}:{:02}", gap_time / 3600 as u32, gap_time % 3600 / 60 as u32 , gap_time % 60 as u32);
        }
        if self.tikpop == true {
            self.time += 1.0;
            frame.set_mouse_passthrough(false);
            if self.time < 50.0 {
                let mut add_x = (self.time / 100.0 * std::f32::consts::PI).sin() * 320.0;
                if self.pos_dir == "right" {
                    add_x = -add_x;
                }
                frame.set_window_pos(Pos2::new(self.init_x + add_x, self.init_y));
            } else if self.time > self.show_time + 50.0 && self.time < self.show_time + 100.0 {
                let mut add_x = ((self.time - self.show_time - 50.0) / 100.0 * std::f32::consts::PI).sin() * 320.0;
                if self.pos_dir != "right" {
                    add_x = self.init_x - add_x + 320.0;
                } else {
                    add_x = self.init_x + add_x - 320.0;
                }
                frame.set_window_pos(Pos2::new(add_x, self.init_y));
            } else if self.time > self.show_time + 100.0 {
                self.tikpop = false;
                self.show_tips = "".to_string();
                self.in_time_popup = false;
                self.visible = self.last_visible;
                frame.set_visible(self.visible);
                if self.visible == true {
                    frame.set_window_pos(Pos2::new(self.last_pos_x, self.last_pos_y));
                }
                frame.set_mouse_passthrough(true);
            }
            if self.visible == false {
                self.tikpop = false;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        } else {
            self.in_time_popup = false;
            let hour = self.now.hour().to_string();
            let minute = self.now.minute().to_string();
            let second = self.now.second().to_string();
            if self.time2show != "" {
                let time2show_arr: Vec<&str> = self.time2show.split(',').collect();
                let mut index:i32 = 0;
                for x in &time2show_arr {
                    let single_time: Vec<&str> = x.split(':').collect();
                    if (single_time[0] == "" || single_time[0] == hour || single_time[0] == "0".to_string() + &hour) &&
                    (single_time[1] == "" || single_time[1] == minute || single_time[1] == "0".to_string() + &minute) &&
                    ((single_time[2] == "" && second == "0") || single_time[2] == second || single_time[2] == "0".to_string() + &second) {
                        if self.tikpop == false {
                            self.in_time_popup = true;
                            begin_tik(index.try_into().unwrap(), self.in_time_popup);
                            self.tikpop = true;
                        }
                        break;
                    }
                    index = index + 1;
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }

        if self.visible == true {
            clock_window_frame(ctx, frame, self, custom_clock);
        }

        if let Ok(TrayEvent {
            event: tray_icon::ClickEvent::Left,
            ..
        }) = tray_icon::TrayEvent::receiver().try_recv()
        {
            self.visible = !self.visible;
            frame.set_visible(self.visible);
            self.tikpop = false;
            self.time = 0.0;
            if self.visible == true {
                frame.set_window_pos(Pos2::new(0.0, self.init_y));
                frame.set_mouse_passthrough(true);
            } else {
                if let Some(pos) = frame.get_window_pos() {
                    self.last_pos_x = pos.x;
                    self.last_pos_y = pos.y;
                }
            }
            ctx.request_repaint();
        }
        if self.show_settings {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .default_width(420.0)
                .show(ctx, |ui| {
                    ui.label("Reminder times (example: :30:,:00:)");
                    ui.text_edit_singleline(&mut self.time2show);
                    ui.add_space(6.0);

                    ui.label("Countdown sequence (example: ::30,::10)");
                    ui.text_edit_singleline(&mut self.countdown);
                    ui.add_space(6.0);

                    ui.label("Tips text");
                    ui.text_edit_singleline(&mut self.tips_store);
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Show time (ms)");
                        ui.text_edit_singleline(&mut self.show_time_input);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Position");
                        egui::ComboBox::from_id_source("pos_dir")
                            .selected_text(self.pos_dir.clone())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.pos_dir, "left".to_string(), "left");
                                ui.selectable_value(&mut self.pos_dir, "right".to_string(), "right");
                            });
                        ui.label("Percent");
                        ui.text_edit_singleline(&mut self.pos_pc_input);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Timezone offset");
                        ui.text_edit_singleline(&mut self.timezone_input);
                        ui.label("blank = local time");
                    });

                    ui.checkbox(&mut self.round, "Rounded corners");
                    ui.add_space(8.0);

                    if self.settings_status != "" {
                        ui.label(&self.settings_status);
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            match self.save_settings() {
                                Ok(()) => self.settings_status = "Saved to conf.ini".to_string(),
                                Err(err) => self.settings_status = err,
                            }
                        }
                        if ui.button("Close").clicked() {
                            self.show_settings = false;
                            self.settings_status = "".to_string();
                        }
                    });
                });
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_index {
                std::process::exit(0)
            } else if event.id == self.settings_index {
                self.show_settings = true;
                self.settings_status = "".to_string();
                self.visible = true;
                frame.set_visible(true);
                ctx.request_repaint();
            } else if event.id == self.countdown_index {
                self.countdown_start = !self.countdown_start;
                if self.countdown_start == true {
                    self.visible = true;
                    frame.set_visible(self.visible);
                    self.countdown_start_time = self.now.timestamp();
                }
                ctx.request_repaint();
            }
        }
    }
}

fn gene_color(color_str: String, default_color: Color32) -> Color32 {
    if color_str == "" {
        return default_color
    }
    let color_arr: Vec<&str> = color_str.split(',').collect();
    if color_arr.len() < 3 {
        return Color32::from_rgb(0, 0, 0)
    }
    if color_arr.len() == 3 {
        return Color32::from_rgb(color_arr[0].to_string().parse::<u8>().unwrap(), 
            color_arr[1].to_string().parse::<u8>().unwrap(), 
            color_arr[2].to_string().parse::<u8>().unwrap())
    }
    return Color32::from_rgba_unmultiplied(color_arr[0].to_string().parse::<u8>().unwrap(), 
        color_arr[1].to_string().parse::<u8>().unwrap(), 
        color_arr[2].to_string().parse::<u8>().unwrap(),  
        color_arr[3].to_string().parse::<u8>().unwrap())
}

fn clock_window_frame(
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    app: &mut POPAlarm,
    custom_clock: String
) {
    use egui::*;
    let text_color = ctx.style().visuals.text_color();
    let panel_fill = gene_color(
        app.custom_bg_color.to_owned(),
        Color32::from_rgba_premultiplied(97, 122, 131, 220),
    );
    let border_color = gene_color(
        app.custom_border_color.to_owned(),
        Color32::from_rgba_premultiplied(222, 231, 236, 190),
    );
    let accent_fill = gene_color(
        app.custom_clock_bg_color.to_owned(),
        Color32::from_rgba_premultiplied(232, 233, 236, 240),
    );
    let time_fill = gene_color(
        app.custom_number_bg_color.to_owned(),
        Color32::from_rgba_premultiplied(40, 49, 58, 225),
    );
    let time_color = gene_color(app.custom_number_color.to_owned(), text_color);

    CentralPanel::default()
        .frame(Frame::none())
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let round = if app.round { 22.0 } else { 14.0 };
            let shadow_rect = rect.translate(vec2(0.0, 6.0)).shrink2(vec2(8.0, 6.0));
            ui.painter().rect_filled(
                shadow_rect,
                round,
                Color32::from_rgba_premultiplied(0, 0, 0, 42),
            );

            let shell_rect = rect.shrink2(vec2(6.0, 6.0));
            ui.painter().rect_filled(shell_rect, round, panel_fill);
            ui.painter().rect_stroke(shell_rect, round, Stroke::new(1.0, border_color));

            let inner_rect = shell_rect.shrink2(vec2(3.0, 3.0));
            ui.painter().rect_filled(
                inner_rect,
                round - 2.0,
                Color32::from_rgba_premultiplied(236, 239, 241, 34),
            );

            let badge_rect = Rect::from_min_max(Pos2::new(20.0, 16.0), Pos2::new(116.0, 88.0));
            ui.painter().rect_filled(
                badge_rect.translate(vec2(0.0, 3.0)),
                18.0,
                Color32::from_rgba_premultiplied(0, 0, 0, 34),
            );
            ui.painter().rect_filled(badge_rect, 18.0, accent_fill);
            ui.painter().rect_stroke(
                badge_rect,
                18.0,
                Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 116)),
            );
            ui.painter().rect_stroke(
                badge_rect.shrink(2.5),
                15.0,
                Stroke::new(1.0, Color32::from_rgba_premultiplied(126, 138, 146, 82)),
            );
            ui.painter().rect_filled(
                Rect::from_min_max(
                    badge_rect.min + vec2(5.0, 5.0),
                    Pos2::new(badge_rect.max.x - 5.0, badge_rect.center().y - 2.0),
                ),
                13.0,
                Color32::from_rgba_premultiplied(255, 255, 255, 42),
            );

            let badge_center = badge_rect.center();
            ui.painter().circle_filled(
                badge_center + vec2(0.0, 1.0),
                29.0,
                Color32::from_rgba_premultiplied(255, 255, 255, 110),
            );
            ui.painter().circle_filled(
                badge_center,
                23.0,
                Color32::from_rgba_premultiplied(244, 245, 247, 148),
            );
            ui.painter().circle_stroke(
                badge_center,
                29.0,
                Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 90)),
            );
            ui.painter().circle_stroke(
                badge_center,
                23.0,
                Stroke::new(1.0, Color32::from_rgba_premultiplied(120, 128, 136, 56)),
            );
            ui.painter().circle_filled(
                badge_center + vec2(-6.0, -8.0),
                10.0,
                Color32::from_rgba_premultiplied(255, 255, 255, 72),
            );

            if let Ok(image) = &app.image {
                let mut size = image.size_vec2();
                size *= 58.0 / size.y.max(1.0);
                let img_rect = Rect::from_center_size(badge_center, size);
                let mut img_ui = ui.child_ui(img_rect, *ui.layout());
                image.show_size(&mut img_ui, size);
            } else {
                let left_ear = badge_center + vec2(-18.0, -13.0);
                let right_ear = badge_center + vec2(18.0, -13.0);
                for ear in [left_ear, right_ear] {
                    ui.painter().text(
                        ear + vec2(1.2, -1.2),
                        Align2::CENTER_CENTER,
                        "P",
                        FontId::proportional(16.0),
                        Color32::from_rgba_premultiplied(255, 255, 255, 126),
                    );
                    ui.painter().text(
                        ear,
                        Align2::CENTER_CENTER,
                        "P",
                        FontId::proportional(16.0),
                        Color32::from_rgba_premultiplied(74, 84, 92, 235),
                    );
                }
                ui.painter().text(
                    badge_center + vec2(1.6, -1.2),
                    Align2::CENTER_CENTER,
                    "O",
                    FontId::proportional(38.0),
                    Color32::from_rgba_premultiplied(255, 255, 255, 118),
                );
                ui.painter().text(
                    badge_center,
                    Align2::CENTER_CENTER,
                    "O",
                    FontId::proportional(38.0),
                    Color32::from_rgba_premultiplied(52, 59, 64, 245),
                );
            }

            ui.painter().text(
                Pos2::new(136.0, 18.0),
                Align2::LEFT_TOP,
                "POPAlarm",
                FontId::proportional(13.0),
                Color32::from_rgba_premultiplied(244, 246, 248, 120),
            );
            ui.painter().text(
                Pos2::new(136.0, 17.0),
                Align2::LEFT_TOP,
                "POPAlarm",
                FontId::proportional(13.0),
                Color32::from_rgba_premultiplied(58, 68, 75, 214),
            );
            ui.painter().text(
                Pos2::new(136.0, 31.0),
                Align2::LEFT_TOP,
                "Hourly reminder",
                FontId::proportional(10.5),
                Color32::from_rgba_premultiplied(229, 235, 239, 80),
            );
            ui.painter().text(
                Pos2::new(136.0, 30.0),
                Align2::LEFT_TOP,
                "Hourly reminder",
                FontId::proportional(10.5),
                Color32::from_rgba_premultiplied(87, 97, 104, 192),
            );

            let time_rect = Rect::from_min_max(
                Pos2::new(132.0, 42.0),
                Pos2::new(rect.right() - 16.0, 76.0),
            );
            ui.painter().rect_filled(time_rect, 11.0, time_fill);
            ui.painter().rect_stroke(
                time_rect,
                11.0,
                Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 30)),
            );

            let shown_clock = if custom_clock == "" {
                app.now.format("%H:%M:%S").to_string()
            } else {
                custom_clock
            };
            ui.painter().text(
                time_rect.center(),
                Align2::CENTER_CENTER,
                shown_clock,
                FontId::monospace(28.0),
                time_color,
            );

            if app.show_tips != "" {
                let tips_rect = Rect::from_min_max(
                    Pos2::new(136.0, 82.0),
                    Pos2::new(rect.right() - 16.0, 95.0),
                );
                ui.painter().text(
                    tips_rect.left_center(),
                    Align2::LEFT_CENTER,
                    &app.show_tips,
                    FontId::proportional(13.0),
                    Color32::from_rgba_premultiplied(64, 73, 79, 220),
                );
            }

            let title_bar_response = ui.interact(rect, Id::new("title_bar"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }

            if app.tikpop == false {
                let close_response = ui.put(
                    Rect::from_min_size(
                        Pos2::new(rect.right() - 32.0, rect.top() + 6.0),
                        Vec2::splat(24.0),
                    ),
                    Button::new(RichText::new("×").size(14.0).color(Color32::from_rgba_premultiplied(58, 68, 75, 200))).frame(false),
                );
                if close_response.clicked() {
                    frame.set_visible(false);
                    app.visible = false;
                }
            }
        });
}