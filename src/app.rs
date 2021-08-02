extern crate egui;
extern crate epi;

mod vertslide;
#[derive(Debug, Copy, Clone)]
struct Vertex {
    used: bool,
    just_released: bool,
    selected: bool,
    
    x: f64,
    y: f64,
    last_x: f64,
    last_y: f64, 
    momentum_x: f64,
    momentum_y: f64,
    momentum_c: f64,
    radius: u32, //should it be i64?
    wheel: bool,
    heading: f64,
    phase: u8
}
#[derive(Debug, Copy, Clone)]
struct Link {
    //used: bool,
    src: usize,
    dest: usize,
    length: f64,
    tension: f64,
    push_timing: i32,
    push_span: i32,
    push_strength: f64 ,
    push: f64, 
    last_len: f64,
    mid_x : f64,
    mid_y : f64,
    phase : u8
}
struct DisplayOptions {
    show_link_handles: bool,
    show_links: bool,
    show_vertices: bool,
    show_wheels: bool,
    num_wheel_spokes: u8,
    shade_wheels: bool,
    shade_body: bool,
    zoom2x: bool,
    shade_color: Color32,
    link_color:Color32,
    link_handle_color: Color32,
    vertex_color:Color32,
    background_color:Color32,
    selection_color:Color32,
    wheel_color:Color32,
    wheel_shade_color:Color32,
    wheel_spoke_color:Color32,
    hover_color:Color32,
}
struct Environment {
    gravity : f64,
    atmosphere : f64,
    wall_bounce : f64,
    wall_friction : f64,
    left_wind : f64,
    tension : f64,
    clock_speed : i32,
}
struct Walls{
    ceiling: bool,
    floor: bool,
    left: bool,
    right: bool,
}
pub struct State {
    display_options: DisplayOptions,
    environment: Environment,
    triangles_updated: bool,
    triangles: std::collections::HashSet<(usize,usize,usize)>,
    walls: Walls,
    vertices: Vec<Vertex>,
    links: Vec<Link>,
    mouse_x: f64,
    mouse_y: f64,
    cycle_time: i32,
    clock_pause: bool,
    mode: u8, //sum type
    sub_mode: u8, //sum type
    sub_mode_data: usize, // todo: improve (use a sum type)
    sel_vertex: Option<usize>,
    sel_link: Option<usize>,
    hover_vertex: Option<usize>,
    hover_link: Option<usize>,
    drag_dot: Option<usize>,
    width: u32,
    height: u32,
    rightwall: f64,
    ceiling: f64,
    auto_reverse_cycle: i32, 
    auto_reverse_enabled: bool,
    current_phase: u8, //sum type?
}
use egui::color::Color32;

#[derive(Debug, Copy, Clone)]
pub enum Presets {
    Walker, Unicycle, Jumper, Spikeball, Muscles, Dancer, AntiGrav, Blank
}
impl State {
    pub fn load_presets(&mut self, preset:Presets) {
        match preset {
            Presets::Blank => self.legacy_parse(include_str!("../blank.botz")).unwrap(),
            Presets::Walker => self.legacy_parse(include_str!("../walker.botz")).unwrap(),
            Presets::Unicycle => self.legacy_parse(include_str!("../unicycle.botz")).unwrap(),
            Presets::Jumper => self.legacy_parse(include_str!("../jumper.botz")).unwrap(),
            Presets::Spikeball => self.legacy_parse(include_str!("../spikeball.botz")).unwrap(),
            Presets::Muscles => self.legacy_parse(include_str!("../muscles.botz")).unwrap(),
            Presets::Dancer => self.legacy_parse(include_str!("../dancer.botz")).unwrap(),
            Presets::AntiGrav => self.legacy_parse(include_str!("../antigrav.botz")).unwrap(),
        }
    }
    pub fn legacy_parse(&mut self,file:&str) -> Option<()> {
        self.vertices = Vec::new();
        self.links = Vec::new();
        self.sel_link = None;
        self.sel_vertex = None;
        self.drag_dot = None;
        self.sub_mode = 0;
        self.triangles_updated = true;
        self.mode = 0;
        let records = file.split(";");
        for i in records {
            if let Some(c) = i.chars().nth(0) {
                if c == 'G' { self.environment.gravity = i[1..].parse().ok()? }
                if c == 'A' { self.environment.atmosphere = i[1..].parse().ok()? }
                if c == 'F' { self.environment.wall_friction = i[1..].parse().ok()? }
                if c == 'B' { self.environment.wall_bounce = i[1..].parse().ok()? }
                if c == 'W' { self.environment.left_wind = i[1..].parse().ok()? }
                if c == 'T' { self.environment.tension = i[1..].parse().ok()? }
                if c == 'C' { self.environment.clock_speed = i[1..].parse().ok()? }
                if c == 'M' { self.mode = i[1..].parse().ok()? }
                if c == 'V' {
                    let mut vertex = Vertex { x:0.0,y:0.0,heading: 0.0, just_released: false, last_x: 0.0, last_y: 0.0, momentum_c: 0.0, momentum_x: 0.0, momentum_y: 0.0, phase:0, radius:0,selected:false,used:true,wheel:false};
                    let subrecords = i[1..].split("|");
                    for j in subrecords {
                        if let Some(c) = j.chars().nth(0) {
                            if c == 'X' { vertex.x = j[1..].parse().ok()? };
                            if c == 'Y' { vertex.y = j[1..].parse().ok()? };
                            if c == 'H' { vertex.momentum_x = j[1..].parse().ok()? };
                            if c == 'U' { vertex.momentum_y = j[1..].parse().ok()? };
                            if c == 'R' { vertex.radius = j[1..].parse().ok()?; vertex.wheel = vertex.radius > 0; };
                            if c == 'C' { vertex.momentum_c = j[1..].parse().ok()? };
                            if c == 'P' { vertex.phase = j[1..].parse().ok()? };
                        }
                    }
                    self.vertices.push(vertex)
                }
                if c == 'L' {
                    let mut link = Link { src: 0, dest: 0, last_len: 0.0,length:0.0,mid_x:0.0,mid_y:0.0,phase:0,push:0.0,push_span:0,push_strength:0.0,push_timing:0,tension:0.9};
                    let subrecords = i[1..].split("|");
                    for j in subrecords {
                        if let Some(c) = j.chars().nth(0) {
                            if c == 'A' { link.src = j[1..].parse().ok()?; link.src -= 1; };
                            if c == 'B' { link.dest = j[1..].parse().ok()?; link.dest -= 1; };
                            if c == 'L' { link.length = j[1..].parse().ok()? };
                            if c == 'T' { link.tension = j[1..].parse().ok()? };
                            if c == 'S' { link.push_span = j[1..].parse().ok()? };
                            if c == 'P' { link.push = j[1..].parse().ok()? };
                            if c == 'N' { link.push_strength = j[1..].parse().ok()? };
                            if c == 'E' { link.last_len = j[1..].parse().ok()? };
                            if c == 'M' { link.push_timing = j[1..].parse().ok()? };
                            //if c == 'P' { link.phase = j[1..].parse().ok()? };
                        }
                    }
                    self.links.push(link)
                }
            }
        }
        
        Some(())
        

    }
    fn mouse_up(&mut self, button2: bool) {
        let howmany = self.how_many_selected();
        if  !button2 { 
            if self.mode == 0 && self.sub_mode == 2 && howmany == 1 { self.sub_mode = 1 };
            if self.mode == 0 && self.sub_mode == 2 && howmany > 1 { self.sub_mode = 0 };
            if self.mode == 1 && self.sub_mode == 2 { self.sub_mode = 0; self.vertices[self.sub_mode_data].just_released = true };
        }
        self.drag_dot = None;
    }
    fn mouse_down(&mut self, button2 : bool, shift: bool) {
        if button2 {
            self.sub_mode = 0;
            self.clear_multi_select();
            self.sel_vertex = None;
            self.sub_mode_data = 0;
            self.drag_dot = None;
            return
        }
        if self.mode == 1 { //simulate
            self.clear_multi_select();
            self.sel_vertex = self.hover_vertex;
            if let Some(i) = self.sel_vertex {
                self.vertices[i].selected = true;
                self.sub_mode_data = i;
                self.sub_mode = 2;
                self.drag_dot = Some(i);
                self.sel_link = None;
            }
        } else if self.mode == 0 {
            if !shift {
                if self.sub_mode == 1 { // continuing a shape?
                    if self.hover_link == None {
                        if self.hover_vertex == None {
                            self.clear_multi_select();
                            let inty = self.add_vertex(self.mouse_x, self.mouse_y, 0.0, 0.0, 0, 0.0, self.current_phase);
                            let _ = self.add_link(inty, self.sub_mode_data);
                            self.sub_mode_data = inty;
                            self.sel_vertex = Some(inty);
                            self.vertices[inty].selected = true;
                            return
                        } else if let Some(inty) = self.hover_vertex {
                            
                            self.clear_multi_select();
                            if !self.add_link(inty, self.sub_mode_data) {
                                self.sub_mode = 2;
                                self.sub_mode_data = inty;
                                self.sel_vertex = Some(inty);
                                self.vertices[inty].selected = true;
                                self.drag_dot = Some(inty);
                                return
                            };
                            self.sub_mode_data = inty;
                            self.sel_vertex = Some(inty);
                            self.vertices[inty].selected = true;
                            return
                        }
                    }
                } else if self.sub_mode == 0 &&  //starting a shape?
                     self.hover_link.is_none() && self.hover_vertex.is_none() {
                        self.clear_multi_select();
                        let inty = self.add_vertex(self.mouse_x, self.mouse_y, 0.0, 0.0, 0, 0.0, self.current_phase);
                        self.sub_mode_data = inty;
                        self.sub_mode = 1;
                        self.sel_vertex = Some(inty);
                        self.vertices[inty].selected = true;
                        return
                    
                } else 
                if let Some(applies) = self.hover_vertex {
                    self.clear_multi_select();
                    self.sel_vertex = Some(applies);
                    self.vertices[applies].selected = true;
                    self.sub_mode_data = applies;
                    self.sub_mode = 2;
                    self.drag_dot = Some(applies);
                    self.sel_link = None;
                }
            }
        }

        if shift && self.mode == 0 {
            if let Some(i) = self.hover_vertex  {
                self.toggle_selection(i);
            }
        } else {
            if let Some(applies) = self.hover_vertex {
                self.sel_vertex = Some(applies);
                self.vertices[applies].selected = true;
                self.sub_mode_data = applies;
                self.sub_mode = 2;
                self.drag_dot = Some(applies);
                self.sel_link = None;
            } else if let Some(applieslink) = self.hover_link {
                self.sel_link = Some(applieslink);
                self.drag_dot = None;
                self.sel_vertex = None;
                self.sub_mode_data = 0;
                self.sub_mode = 4;
                self.clear_multi_select();
            }
        }
    }
    fn mouse_move(&mut self, x : f32, y : f32) {
        self.mouse_x = x  as f64  / if self.display_options.zoom2x {2.0} else {1.0};
        self.mouse_y = (self.height as f64 - y as f64)  / if self.display_options.zoom2x {2.0} else {1.0} ;
        if self.sub_mode == 2 {
            self.vertices[self.sub_mode_data].x = self.mouse_x;
            self.vertices[self.sub_mode_data].y = self.mouse_y;
        }
        for i in 0..self.vertices.len() {
            if self.vertices[i].used {
                if self.mouse_x > (self.vertices[i].x - 12.0) && self.mouse_x < (self.vertices[i].x + 12.0) {
                    if self.mouse_y > (self.vertices[i].y - 12.0) && self.mouse_y < (self.vertices[i].y + 12.0) {
                        self.hover_vertex = Some(i);
                        self.hover_link = None;
                        return
                    }   
                }
            }
            for i in 0..self.links.len() {
                if self.mouse_x > (self.links[i].mid_x - 12.0) && self.mouse_x < (self.links[i].mid_x + 12.0) {
                    if self.mouse_y > (self.links[i].mid_y - 12.0) && self.mouse_y < (self.links[i].mid_y + 12.0) {
                        self.hover_vertex = None;
                        self.hover_link = Some(i);
                        return
                    }
                }
            }
            self.hover_vertex = None;
            self.hover_link = None;
        }
    }
    fn cycle_physics(&mut self) {
        let cycle_size = 200;
        if !self.clock_pause  {
            self.cycle_time += self.environment.clock_speed;
            while self.cycle_time > cycle_size { self.cycle_time -= cycle_size };
            while self.cycle_time < 0  { self.cycle_time += cycle_size}
        }
        for i in 0..self.links.len() {
            let link = &mut self.links[i];
            link.push = 0.0;
            if self.cycle_time >= link.push_timing - link.push_span && self.cycle_time < link.push_timing + link.push_span {
                link.push = link.push_strength * (1.0 - ((link.push_timing - self.cycle_time).abs() as f64 / (link.push_span as f64)));
                link.push = (link.push / 30.0) * link.length;
            }
            if link.push_timing + link.push_span > cycle_size && self.cycle_time < link.push_timing + link.push_span - cycle_size {
                let temp = link.push_timing - cycle_size;
                link.push = link.push_strength * (1.0 - (((temp - self.cycle_time).abs() as f64 / link.push_span as f64)));
                link.push = (link.push / 30.0) * link.length;
            }
            if link.push_timing - link.push_span < 0 && self.cycle_time > link.push_timing - link.push_span + cycle_size {
                let temp = link.push_timing + cycle_size;
                link.push = link.push_strength * (1.0 - (((temp - self.cycle_time).abs() as f64 / link.push_span as f64)));
                link.push = (link.push / 30.0) * link.length;
            }
            let length_total = if self.clock_pause { link.length } else { link.length + link.push };
            let t1 = self.vertices[link.src] ;
            let t2 = self.vertices[link.dest];
            let xer = (t2.x + t2.momentum_x) - (t1.x + t1.momentum_x);
            let yer = (t2.y + t2.momentum_y) - (t1.y + t1.momentum_y);
            let leng = (xer * xer + yer * yer).abs().sqrt();
            let leng2go_x = ((leng - length_total) / leng) * xer;
            let leng2go_y = ((leng - length_total) / leng) * yer;
            { 
                let t1 = & mut self.vertices[link.src];
                t1.momentum_x = t1.momentum_x + (leng2go_x / 2.0) * link.tension;
                t1.momentum_y = t1.momentum_y + (leng2go_y / 2.0) * link.tension;
            }
            {
                let t2 = & mut self.vertices[link.dest];
                t2.momentum_x = t2.momentum_x + (leng2go_x / 2.0) * -1.0 * link.tension;
                t2.momentum_y = t2.momentum_y + (leng2go_y / 2.0) * -1.0 * link.tension;
            }
        }
        for i in 0..self.vertices.len() {
            let vertex = &mut self.vertices[i];
            if !vertex.used { continue; }
            vertex.momentum_y -= self.environment.gravity * 1.5;
            if vertex.just_released { vertex.momentum_x = 0.0; vertex.momentum_y = 0.0; vertex.just_released = false }
            vertex.momentum_x += self.environment.left_wind / 10.0;
            vertex.momentum_x *= 1.0 - self.environment.atmosphere;
            vertex.momentum_y *= 1.0 - self.environment.atmosphere;
            if self.drag_dot == Some(i) && self.sub_mode == 2 {
                vertex.momentum_x = 0.0;
                vertex.momentum_y = 0.0;
            }
            vertex.last_x = vertex.x;
            vertex.last_y = vertex.y;
            vertex.x += vertex.momentum_x;
            vertex.y += vertex.momentum_y;
            // this shouldn't be needed            
            vertex.wheel = vertex.radius > 0;
            self.ceiling = (self.height as f64 - 4.0) / if self.display_options.zoom2x {2.0} else {1.0};
            self.rightwall = (self.width as f64 - 4.0)  / if self.display_options.zoom2x {2.0} else {1.0};
            let fric = if vertex.wheel { 0.0 } else { self.environment.wall_friction };
            // TODO: enable options to toggle walls
            if self.walls.floor && vertex.y - (vertex.radius as f64) < 0.1 { // floor
                
                vertex.y = vertex.radius as f64;
                vertex.momentum_x *= 1.0 - fric;
                vertex.momentum_y = (vertex.momentum_y * self.environment.wall_bounce) * -1.0;
                if vertex.wheel { vertex.momentum_c = vertex.momentum_x }
            }
            if self.walls.left && vertex.x - (vertex.radius as f64) < 0.1 { // left wall
                vertex.x = vertex.radius as f64;
                vertex.momentum_y *= 1.0 - fric;
                vertex.momentum_x = (vertex.momentum_x * self.environment.wall_bounce) * -1.0;
                if vertex.wheel { vertex.momentum_c = -vertex.momentum_y }
                if self.auto_reverse_enabled {
                    if self.auto_reverse_cycle == 0 { self.auto_reverse_cycle = 2; self.environment.clock_speed *= -1; };
                    if self.auto_reverse_cycle == 1 { self.auto_reverse_cycle = 2; self.environment.clock_speed *= -1; };
                }
            }
            if self.walls.right && vertex.x + (vertex.radius as f64) > self.rightwall - 0.1 { // right wall
                vertex.x = self.rightwall - vertex.radius as f64;
                vertex.momentum_y *= 1.0 - fric;
                vertex.momentum_x = (vertex.momentum_x * self.environment.wall_bounce) * -1.0;
                if vertex.wheel { vertex.momentum_c = vertex.momentum_y }
                if self.auto_reverse_enabled {
                    if self.auto_reverse_cycle == 0 { self.auto_reverse_cycle = 1; self.environment.clock_speed *= -1; };
                    if self.auto_reverse_cycle == 2 { self.auto_reverse_cycle = 1; self.environment.clock_speed *= -1; };
                }
            }
            if self.walls.ceiling && vertex.y + (vertex.radius as f64) > self.ceiling - 0.1 {
                vertex.y = self.ceiling - vertex.radius as f64;
                vertex.momentum_x *= 1.0 - fric;
                vertex.momentum_y = (vertex.momentum_y * self.environment.wall_bounce) * -1.0;
                if vertex.wheel { vertex.momentum_c = -vertex.momentum_x }
            }
            vertex.heading += vertex.momentum_c;
            if vertex.heading > 360.0 { vertex.heading -= 360.0 };
            if vertex.heading < 0.0 { vertex.heading += 360.0 };
        }
        
    }
    fn draw_playfield_line(&self,ui: &mut egui::Ui,  rect: &egui::Rect, x: f64, y: f64, tx: f64, ty: f64, color: Color32) {
        if self.display_options.zoom2x {
            ui.painter().line_segment([rect.left_top() + egui::Vec2::new(2.0*x as f32, rect.height() - (2.0 * y) as f32), rect.left_top() + egui::Vec2::new(2.0*tx as f32, rect.height() - (2.0 * ty) as f32)], egui::Stroke::new(2.0,color) );
        } else {
            ui.painter().line_segment([rect.left_top() + egui::Vec2::new(1.0*x as f32, rect.height() - (1.0 * y) as f32), rect.left_top() + egui::Vec2::new(1.0*tx as f32, rect.height() - (1.0 * ty) as f32)], egui::Stroke::new(1.0,color) );
        }
    }
    fn draw_playfield_dot(&self,ui: &mut egui::Ui,   rect: &egui::Rect,x: f64, y: f64, color: Color32) {
        if self.display_options.zoom2x {
            ui.painter().circle_filled(rect.left_top() + egui::Vec2::new(2.0*x as f32, rect.height() - (2.0 * y) as f32), 4.0, color);
            
        } else {
            ui.painter().circle_filled(rect.left_top() + egui::Vec2::new(1.0*x as f32, rect.height() - (1.0 * y) as f32), 2.0, color);
        }
    }
    fn draw_playfield_circle(&self, ui: &mut egui::Ui,  rect: &egui::Rect, x: f64, y: f64, r: u32, color: Color32) {
        ui.painter().circle_stroke(self.to_playfield(rect, x,y), r as f32,  egui::Stroke::new(if self.display_options.zoom2x { 2.0 } else { 1.0 },color));
    }
    fn draw_playfield_filled_circle(&self, ui: &mut egui::Ui,  rect: &egui::Rect, x: f64, y: f64, r: u32, color: Color32) {
        ui.painter().circle_filled(self.to_playfield(rect, x,y), r as f32, color);
    }
    fn to_playfield(&self, rect: &egui::Rect,x:f64,y:f64) -> egui::Pos2 {
        if self.display_options.zoom2x {
            rect.left_top() + egui::Vec2::new(2.0*x as f32, rect.height() - (2.0 * y) as f32)
        } else {
            rect.left_top() + egui::Vec2::new(1.0*x as f32, rect.height() - (1.0 * y) as f32)
        }
    }
    fn draw(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        
        if self.display_options.show_links {
            for i in 0..self.links.len() {
                let link = &mut self.links[i];
                let src = self.vertices[link.src];
                let dest = self.vertices[link.dest];
                link.mid_x = dest.x + ((src.x - dest.x) / 2.0);
                link.mid_y = dest.y + ((src.y - dest.y) / 2.0);
                let link = & self.links[i];
                let color1 = if self.sel_link == Some(i) {
                    self.display_options.selection_color
                } else {
                    self.display_options.link_handle_color
                };
                
                
                self.draw_playfield_line(ui,&rect,src.x, src.y, dest.x, dest.y,self.display_options.link_color);
                if self.display_options.show_link_handles {
                    self.draw_playfield_line( ui,&rect,link.mid_x -5.0, link.mid_y, link.mid_x + 5.0, link.mid_y,color1);
                    self.draw_playfield_line( ui,&rect,link.mid_x, link.mid_y-5.0, link.mid_x , link.mid_y+ 5.0,color1);
                }
                if self.hover_link == Some(i) {
                    self.draw_playfield_circle(ui,&rect, link.mid_x, link.mid_y, 4, self.display_options.hover_color);
                } else if self.sel_link == Some(i) {
                    self.draw_playfield_circle(ui,&rect, link.mid_x, link.mid_y, 4, self.display_options.selection_color);
                }
                
            }
            if self.mode == 0 && self.sub_mode == 1 {
                if let Some (i) = self.sel_vertex {
                    if !ui.input().modifiers.shift {
                        self.draw_playfield_line(ui,&rect, self.vertices[i].x, self.vertices[i].y, self.mouse_x, self.mouse_y, Color32::GRAY);
                    }
                }
            }
        }
        
        for i in 0..self.vertices.len() {
            let vertex = & self.vertices[i];
            if !vertex.used { continue};
            if self.display_options.show_vertices {
                self.draw_playfield_dot(ui,&rect,vertex.x, vertex.y, self.display_options.vertex_color);
            }
            if self.hover_vertex == Some(i) {
                self.draw_playfield_circle( ui,&rect,vertex.x, vertex.y, 4, self.display_options.hover_color);
            }
            if vertex.selected {
                self.draw_playfield_circle(ui,&rect, vertex.x, vertex.y, 4, self.display_options.selection_color);
            }
            if self.display_options.show_wheels && vertex.wheel {
                self.draw_playfield_circle(ui, &rect,vertex.x, vertex.y, vertex.radius, self.display_options.wheel_color);
                if self.display_options.shade_wheels {
                    self.draw_playfield_filled_circle(ui, &rect,vertex.x, vertex.y, vertex.radius, self.display_options.wheel_shade_color);
                }
                let max_spokes = self.display_options.num_wheel_spokes;
                if max_spokes > 0 {
                    for i in 0..max_spokes {
                        let subheading = vertex.heading + ((360.0 / max_spokes as f64) * (i as f64));
                        let x = (subheading*std::f64::consts::PI/180.0).sin() * (vertex.radius as f64/if self.display_options.zoom2x { 2.0 } else { 1.0 });
                        let y = (subheading*std::f64::consts::PI/180.0).cos() * (vertex.radius as f64/if self.display_options.zoom2x { 2.0 } else { 1.0 });
                        self.draw_playfield_line(ui,&rect,vertex.x, vertex.y, vertex.x - x, vertex.y - y, self.display_options.wheel_spoke_color);                        
                    }
                }
            }
        }
        if self.display_options.shade_body {
            self.find_triangles();
            let mut vec = Vec::new();
            for (x1,x2,x3) in &self.triangles {
                let (p1,p2,p3) = (*x1,*x2,*x3);
                vec.push(egui::Shape::convex_polygon(vec![
                    self.to_playfield(&rect, self.vertices[p1].x,self.vertices[p1].y),self.to_playfield(&rect, self.vertices[p2].x,self.vertices[p2].y),
                    self.to_playfield(&rect, self.vertices[p2].x,self.vertices[p2].y),self.to_playfield(&rect, self.vertices[p3].x,self.vertices[p3].y),
                    self.to_playfield(&rect, self.vertices[p3].x,self.vertices[p3].y),self.to_playfield(&rect, self.vertices[p1].x,self.vertices[p1].y),
                ], self.display_options.shade_color, egui::Stroke::new(0.0, Color32::BLACK)))
            }
            ui.painter().extend(vec);
        }
    }
    fn find_triangles(&mut self)   {
        if !self.triangles_updated {
            return
        }
        let mut adj: Vec<Vec<bool>> = vec![vec![false;self.vertices.len()]; self.vertices.len()];
        let mut found : std::collections::HashSet<(usize,usize,usize)> = std::collections::HashSet::new();
        for i in 0..self.links.len() {
            adj[self.links[i].src][self.links[i].dest] = true;
            adj[self.links[i].dest][self.links[i].src] = true;
        }
        for link in &self.links {
            for j in 0..self.vertices.len() {
                if j != link.src && j != link.dest && adj[link.src][j] && adj[link.dest][j] {
                    if j < link.src && link.src < link.dest {
                        found.insert((j,link.src,link.dest));
                    } else if j < link.dest && link.dest < link.src {
                        found.insert((j,link.dest,link.src));
                    } else if link.src < j && j < link.dest {
                        found.insert((link.src,j,link.dest));
                    } else if link.dest < j && j < link.src {
                        found.insert((link.dest,j,link.src));
                    } else if link.dest < link.src && link.src < j {
                        found.insert((link.dest,link.src, j));
                    } else {
                        found.insert((link.src,link.dest, j));
                    }
                }
            }
        }
        self.triangles = found;
        self.triangles_updated = false;
    }
    fn reset_all_links(&mut self) {
        for i in 0..self.links.len() {
            self.reset_link(i);
        }
    }
    fn reset_all_connected_links(&mut self) {
        for i in 0..self.links.len() {
            let t = self.links[i].src;
            if self.vertices[t].used && self.vertices[t].selected {
                self.reset_link(i);
                continue;
            }
            let t = self.links[i].dest;
            if self.vertices[t].used && self.vertices[t].selected {
                self.reset_link(i);
                continue;
            }
        }
    }
    fn reset_link(&mut self, link: usize) {
        let t1 = self.vertices[self.links[link].src];
        let t2 = self.vertices[self.links[link].dest];        
        let xer = t2.x - t1.x;
        let yer = t2.y - t1.y;
        self.links[link].length = ((xer * xer + yer * yer).abs()).sqrt()
    }

    fn set_wheel(&mut self, vertex: usize, radius: u32) {
        self.vertices[vertex].radius = radius;
        self.vertices[vertex].wheel = radius > 0;
        self.vertices[vertex].heading = 0.0;
    }
    fn add_vertex(&mut self, x: f64, y: f64, momentum_x: f64, momentum_y: f64, radius: u32, momentum_c: f64, phase: u8) -> usize {
        
        let vertex = Vertex {
            x,y,momentum_c,momentum_x,momentum_y,radius,phase,
            heading:0.0, wheel: radius > 0, just_released:false, last_x:0.0,last_y:0.0, selected:false,
            used:true,
        };
        for i in 0..self.vertices.len() {
            if self.vertices[i].used == false {
                self.vertices[i] = vertex;
                return i;
            }
        }
        self.triangles_updated = true;
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    fn delete(&mut self) {
        
        if let Some(i) = self.sel_vertex {
            self.delete_vertex(i);
            for i in 0..self.vertices.len() {
                if self.vertices[i].selected {
                    self.delete_vertex(i);
                }
            }
        }

        
        if let Some(i) = self.sel_link {
            self.delete_link(i);
        }
        self.clear_multi_select();
        self.sel_vertex = None;
        self.sel_link = None;
    }
    fn delete_link(&mut self, id: usize) {
        self.links.remove(id);
        self.triangles_updated = true;
    }
    fn delete_vertex(&mut self, id: usize) {
        if self.sub_mode_data == id || self.drag_dot == Some(id) {
            self.sub_mode_data = 0;
            self.sub_mode = 0;
            self.drag_dot = None;
        }
        let mut i = 0;
        while i != self.links.len() {
            if  self.links[i].src == id || self.links[i].dest == id {
                self.links.remove(i);
            } else {
                i += 1;
            }
        }
        self.vertices[id].used = false;
        self.triangles_updated = true;
    }
    fn how_many_selected(&self) -> usize {
        self.vertices.iter().filter(|x| x.selected && x.used).count()
    }
    fn toggle_selection(&mut self, id: usize) {
        self.vertices[id].selected = !self.vertices[id].selected
    }
    fn clear_multi_select(&mut self) {
        for vertex in &mut self.vertices {
            vertex.selected = false;
        }
    }
    fn add_link(&mut self, src: usize, dest: usize) -> bool {
        if src == dest { return false };
        for link in &self.links {
            if link.src == src && link.dest == dest { return false };
            if link.dest == src && link.src == dest { return false };
        }
        let x_len = self.vertices[dest].x - self.vertices[src].x;
        let y_len = self.vertices[dest].y - self.vertices[src].y;
        let len = (x_len * x_len + y_len * y_len).sqrt();
        let link = Link {
            src: src,
            dest: dest,
            tension: self.environment.tension,
            length: len,
            last_len: 0.0,
            phase: self.current_phase,
            push_strength: 0.0,
            push_timing: 180,
            push_span: 40,
            push: 0.0,
            mid_x: self.vertices[dest].x + (self.vertices[src].x - self.vertices[dest].x) / 2.0,
            mid_y: self.vertices[dest].y + (self.vertices[src].y - self.vertices[dest].y) / 2.0,
        };
        
        self.links.push(link);
        self.triangles_updated = true;
        return true;
    }
}
impl epi::App for State {
fn max_size_points(&self) -> egui::Vec2 {
    // Some browsers get slow with huge WebGL canvases, so we limit the size:
    egui::Vec2::new(2048.0, 2048.0)
}
fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) { 
    let mut vis = egui::Visuals::light();
    vis.window_shadow.extrusion = (vis.window_shadow.extrusion + 1.0) / 8.0;
    ctx.set_visuals(vis);
    if self.mode == 1 { self.cycle_physics(); };
    egui::CentralPanel::default().show(ctx, |ui| {
        let rect = ui.max_rect_finite();
        let (_rect2, response) = ui.allocate_exact_size(rect.size(), egui::Sense::click_and_drag());
        if response.clicked() {
            
        }
        if let Some(egui::Pos2 { x,y }) = response.hover_pos() {
            if ui.input().pointer.is_moving() {
                self.mouse_move(x - rect.left(), y - rect.top());    
            }
            if ui.input().pointer.any_pressed() {
                
                self.mouse_down(ui.input().pointer.button_down(egui::PointerButton::Secondary), ui.input().modifiers.shift)
            }
            if ui.input().pointer.any_released() {
                
                self.mouse_up(ui.input().modifiers.shift)
            }

        }
        self.width = rect.width() as u32;
        self.height = rect.height() as u32;
        ui.painter().rect_filled(rect, 4.0, self.display_options.background_color);
        
        self.draw(ui,rect);
       
    });
    egui::Window::new("Menu").title_bar(false).fixed_size(egui::Vec2::new(60.0,100.0))
    .show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(1.0, 2.0);
            if ui.selectable_label(self.mode == 0,"✏️"[0..3].to_string()).on_hover_ui(|ui| {ui.label("Edit");}).clicked() {
                self.mode = 0;
                self.sub_mode = 0;
                self.clear_multi_select();
                self.sel_vertex = None;
                self.sel_link = None;
            };
            if ui.selectable_label(self.mode == 1,"▶").on_hover_ui(|ui| {ui.label("Simulate");}).clicked() {
                self.mode = 1;
                self.sub_mode = 0;
                self.clear_multi_select();
                self.sel_vertex = None;
                self.sel_link = None;
            };
        });
        ui.set_width(200.0);
        ui.collapsing("Display Options", |ui| {  egui::Grid::new("poswtable").show(ui, |ui|{
            
            ui.checkbox(&mut self.display_options.show_links, "Links");
            if self.display_options.show_links {
                ui.add_space(4.0);
                egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.link_color,egui::widgets::color_picker::Alpha::OnlyBlend);
                ui.end_row();
                
                    ui.add_space(8.0);
                    ui.checkbox(&mut self.display_options.show_link_handles, "Link Handles");
                    ui.add_space(-8.0);
                
                if self.display_options.show_link_handles {
                    ui.add_space(4.0);
                    egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.link_handle_color,egui::widgets::color_picker::Alpha::OnlyBlend);
                }
                
                
            }
            ui.end_row();
            
            ui.checkbox(&mut self.display_options.show_vertices, "Vertices");
            if self.display_options.show_vertices {
                ui.add_space(4.0);
                egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.vertex_color,egui::widgets::color_picker::Alpha::OnlyBlend);
            }
            ui.end_row();
            ui.checkbox(&mut self.display_options.show_wheels, "Wheels");
            if self.display_options.show_wheels {
                ui.add_space(4.0);
                egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.wheel_color,egui::widgets::color_picker::Alpha::OnlyBlend);
                ui.end_row();
                
                    ui.add_space(8.0);
                    ui.add(egui::DragValue::new(&mut self.display_options.num_wheel_spokes).speed(1).clamp_range(0..=100).suffix(" spokes"));
                    ui.add_space(-8.0);
                    ui.add_space(4.0);                    
                egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.wheel_spoke_color,egui::widgets::color_picker::Alpha::OnlyBlend);
                ui.end_row();
                
                    ui.add_space(8.0);
                    ui.checkbox(&mut self.display_options.shade_wheels, "Shade Wheels");
                    ui.add_space(-8.0);
                
                if self.display_options.shade_wheels {
                    ui.add_space(4.0);
                    egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.wheel_shade_color,egui::widgets::color_picker::Alpha::OnlyBlend);
                }
                ui.end_row();
            }
            ui.end_row();
            
            ui.checkbox(&mut self.display_options.shade_body, "Body Shading"); 
            if self.display_options.shade_body {
                ui.add_space(4.0);
                egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.shade_color,egui::widgets::color_picker::Alpha::OnlyBlend);
            }
            ui.end_row();
            
            ui.checkbox(&mut self.display_options.zoom2x, "2x Bigger");
            ui.end_row();
            ui.label("Selection colour");
            ui.add_space(4.0);
            egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.selection_color,egui::widgets::color_picker::Alpha::OnlyBlend);
            ui.end_row();
            ui.label("Hover colour");
            ui.add_space(4.0);
            egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.hover_color,egui::widgets::color_picker::Alpha::OnlyBlend);
            ui.end_row();
            ui.label("Background colour");
            ui.add_space(4.0);
            egui::widgets::color_picker::color_edit_button_srgba(ui,&mut self.display_options.background_color,egui::widgets::color_picker::Alpha::OnlyBlend);
        });});
        ui.collapsing("Load Preset", |ui| {
            if ui.button("Blank").clicked() { self.load_presets(Presets::Blank)};
            if ui.button("Walker").clicked() { self.load_presets(Presets::Walker)};
            if ui.button("AntiGrav").clicked() { self.load_presets(Presets::AntiGrav)};
            if ui.button("Dancer").clicked() { self.load_presets(Presets::Dancer)};
            if ui.button("Unicycle").clicked() { self.load_presets(Presets::Unicycle)};
            if ui.button("Jumper").clicked() { self.load_presets(Presets::Jumper)};
            if ui.button("SpikeBall").clicked() { self.load_presets(Presets::Spikeball)};
            if ui.button("Muscles").clicked() { self.load_presets(Presets::Muscles)};    
        });
        if ui.input().key_pressed(egui::Key::Backspace) || ui.input().key_pressed(egui::Key::Delete) {
            self.delete();
        }
        if self.sel_vertex == None && self.sel_link == None {
            egui::CollapsingHeader::new("No selection").default_open(true).show(ui,|ui| {
                if ui.add(egui::Button::new("🔄")).on_hover_ui(|ui| {ui.label("Reset lengths of all links");}).clicked() {
                    self.reset_all_links();
                };
            });
        } else {
            if let Some(n) = self.sel_link {
                egui::CollapsingHeader::new(format!("Link {} selected",n)).default_open(true).show(ui, |ui| {
                    if ui.button("🗑️"[0..4].to_string()).on_hover_ui(|ui| {ui.label("Delete");}).clicked() {
                        self.delete();
                    }
                    egui::Grid::new("postable").show(ui, |ui|{
                        ui.add(egui::Label::new("True Length"));
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.links[n].length).speed(0.5));
                            if ui.add(egui::Button::new("🔄")).on_hover_ui(|ui| {ui.label("Reset true length to actual length");}).clicked() {
                                self.reset_link(n);
                            };
                        });
                        ui.end_row();
                    
                        ui.add(egui::Label::new("Tension"));
                        ui.add(egui::DragValue::new(&mut self.links[n].tension).speed(0.01).clamp_range(0.0..=1.5))
                    });
                });
            } else if let Some(n) = self.sel_vertex {
                if self.how_many_selected() > 1 {
                    egui::CollapsingHeader::new(format!("{} vertices selected",self.how_many_selected())).default_open(true).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("🗑️"[0..4].to_string()).on_hover_ui(|ui| {ui.label("Delete");}).clicked() {
                                self.delete();
                            }
                            if ui.add(egui::Button::new("🔄")).on_hover_ui(|ui| {ui.label("Reset lengths of connected links");}).clicked() {
                                self.reset_all_connected_links();
                            };
                        });
                    });
                } else {
                    egui::CollapsingHeader::new(format!("Vertex {} selected",n)).default_open(true).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("🗑️"[0..4].to_string()).on_hover_ui(|ui| {ui.label("Delete");}).clicked() {
                                self.delete();
                            }
                            if ui.add(egui::Button::new("🔄")).on_hover_ui(|ui| {ui.label("Reset lengths of connected links");}).clicked() {
                                self.reset_all_connected_links();
                            };
                        });
                        egui::Grid::new("postable2").show(ui, |ui|{
                        
                            if self.vertices[n].radius == 0 {
                                ui.label("Wheel");
                                if ui.button("Add ").clicked() {
                                    self.set_wheel(n, 20);
                                }
                                ui.end_row();
                            } else {
                                ui.add(egui::Label::new("Wheel"));
                                if ui.add(egui::DragValue::new(&mut self.vertices[n].radius).speed(0.5)).changed() {
                                    self.vertices[n].wheel = self.vertices[n].radius > 0;
                                };
                                ui.end_row();
                            }                    
                            ui.add(egui::Label::new("X"));
                            ui.add(egui::DragValue::new(&mut self.vertices[n].x).speed(0.5));
                            ui.end_row();
                        
                            ui.add(egui::Label::new("Y"));
                            ui.add(egui::DragValue::new(&mut self.vertices[n].y).speed(0.5))
                        });
                    });
                    
                }
            }
        }
    });
    egui::Window::new("Environment").fixed_size(egui::Vec2::new(40.0,100.0))
    .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.add(vertslide::VSlider::new(&mut self.environment.gravity, -1.0..=1.0).text("Gravity"));
                    
                    ui.vertical(|ui| {
                        ui.add(egui::Label::new("Atmosphere").wrap(false));
                        ui.add(egui::Slider::new(&mut self.environment.atmosphere, 0.0..=1.0));
                        ui.label("Wind");
                        ui.add(egui::Slider::new(&mut self.environment.left_wind, -20.0..=20.0));
                        ui.label("Wall Friction");
                        ui.add(egui::Slider::new(&mut self.environment.wall_friction, -1.0..=2.0));
                        ui.label("Wall Bounce");
                        ui.add(egui::Slider::new(&mut self.environment.wall_bounce, 0.0..=2.0));
                    })
                
                    /*
                        
                    */
                });
                
                if ui.selectable_label(self.walls.ceiling,"Ceiling").clicked() {
                    self.walls.ceiling = !self.walls.ceiling;
                }
                ui.columns(2, |columns| {
                    columns[0].centered_and_justified(|ui| 
                        if ui.selectable_label(self.walls.left,"Wall L").clicked() {
                            self.walls.left = !self.walls.left;
                        });
                    columns[1].centered_and_justified(|ui| if ui.selectable_label(self.walls.right,"Wall R").clicked() {
                        self.walls.right = !self.walls.right;
                    });
                });
                if ui.selectable_label(self.walls.floor,"Floor").clicked() {
                    self.walls.floor = !self.walls.floor;
                }
            });
        
    });
    egui::Window::new("Muscles").fixed_size(egui::Vec2::new(250.0,100.0))
    .show(ctx, |ui| {
        ui.vertical(|ui| {
            let rect = ui.max_rect_finite();
            let (_rect2, response) = ui.allocate_exact_size(rect.size(), egui::Sense::click_and_drag());
            
            ui.painter().rect_filled(rect, 4.0, Color32::WHITE);
            
            ui.painter().line_segment([rect.left_top() + egui::Vec2::new(self.cycle_time as f32 * rect.width() / 200.0,0.0) , rect.left_bottom() + egui::Vec2::new(self.cycle_time as f32 * rect.width() / 200.0,0.0)], egui::Stroke::new(1.0,Color32::GOLD) );
            ui.painter().line_segment([rect.left_top() + egui::Vec2::new(50.0 * rect.width() / 200.0,0.0) , rect.left_bottom() + egui::Vec2::new(50.0 * rect.width() / 200.0,0.0)], egui::Stroke::new(1.0,Color32::LIGHT_GRAY) );
            ui.painter().line_segment([rect.left_top() + egui::Vec2::new(100.0 * rect.width() / 200.0,0.0) , rect.left_bottom() + egui::Vec2::new(100.0 * rect.width() / 200.0,0.0)], egui::Stroke::new(1.0,Color32::LIGHT_GRAY) );
            ui.painter().line_segment([rect.left_top() + egui::Vec2::new(150.0 * rect.width() / 200.0,0.0) , rect.left_bottom() + egui::Vec2::new(150.0 * rect.width() / 200.0,0.0)], egui::Stroke::new(1.0,Color32::LIGHT_GRAY) );
            ui.painter().line_segment([rect.left_top() + egui::Vec2::new(0.0,rect.height()/2.0) , rect.right_top() + egui::Vec2::new(0.0,rect.height()/2.0)], egui::Stroke::new(1.0,Color32::BLACK) );
            for i in 0..self.links.len() {
                let cycle_size = 200;
                
                if self.links[i].push_span > 0 && self.links[i].push_strength != 0.0 && self.links[i].push_timing <= cycle_size {

                    let col = if Some(i) == self.sel_link { Color32::RED } else if Some(i) == self.hover_link { Color32::BLUE } else { Color32::GRAY };
                    let p1 = egui::Vec2::new(self.links[i].push_timing as f32 * rect.width() / cycle_size as f32,(rect.height()/2.0)  - (self.links[i].push_strength as f32 * (rect.height()/2.0)/ 20.0));
                    let p2 = egui::Vec2::new((self.links[i].push_timing as f32 - self.links[i].push_span as f32) * rect.width() / cycle_size as f32,rect.height()/2.0);
                    let p3 = egui::Vec2::new((self.links[i].push_timing as f32 + self.links[i].push_span as f32) * rect.width() / cycle_size as f32,rect.height()/2.0);
                    let p = ui.painter_at(rect);
                
                    p.line_segment([rect.left_top() + p1, rect.left_top() + p2], egui::Stroke::new(1.0,col) );
                    p.line_segment([rect.left_top() + p1, rect.left_top() + p3], egui::Stroke::new(1.0,col) );
                    p.line_segment([rect.left_top() + p1 + egui::Vec2::new(rect.width(),0.0), rect.left_top() + p2 + egui::Vec2::new(rect.width(),0.0)], egui::Stroke::new(1.0,col) );
                    p.line_segment([rect.left_top() + p1 + egui::Vec2::new(rect.width(),0.0), rect.left_top() + p3 + egui::Vec2::new(rect.width(),0.0)], egui::Stroke::new(1.0,col) );
                    p.line_segment([rect.left_top() + p1 - egui::Vec2::new(rect.width(),0.0), rect.left_top() + p2 - egui::Vec2::new(rect.width(),0.0)], egui::Stroke::new(1.0,col) );
                    p.line_segment([rect.left_top() + p1 - egui::Vec2::new(rect.width(),0.0), rect.left_top() + p3 - egui::Vec2::new(rect.width(),0.0)], egui::Stroke::new(1.0,col) );
                }
            }
            if let Some(pos) = response.hover_pos() {
                if self.sel_link.is_some() {
                    ui.painter().line_segment([egui::Pos2::new(rect.left(),pos.y) , egui::Pos2::new(rect.right(),pos.y)], egui::Stroke::new(1.0,Color32::LIGHT_GRAY) );
                    ui.painter().line_segment([egui::Pos2::new(pos.x,rect.top()) , egui::Pos2::new(pos.x,rect.bottom())], egui::Stroke::new(1.0,Color32::LIGHT_GRAY) );
                }
                if rect.contains(pos) {
                    if let Some(spos) = ui.input().pointer.press_origin() {
                        let adj_pos = egui::Pos2::new(spos.x - rect.left(), pos.y - rect.top()) ;

                        let x = 200.0 * adj_pos.x / rect.width();
                        let y = (rect.height() - adj_pos.y - (rect.height()/2.0)) * (20.0)/ (rect.height()/2.0);
                        if let Some(i) = self.sel_link {
                           self.links[i].push_strength = y as f64;
                           self.links[i].push_timing = x as i32;
                           self.links[i].push_span = ((spos.x - pos.x).abs() * 200.0 / rect.width()) as i32;
                           if self.links[i].push_span == 0 && self.links[i].push_strength > 5.0 {
                              self.links[i].push_span = 5;
                           }
                           if self.links[i].push_span > 100 {
                               self.links[i].push_span = 100;
                           }
                        }

                    }
                    
                }
            }
            ui.horizontal(|ui| {
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.environment.clock_speed, -10..=10));
                if ui.button("⏪"[0..3].to_string()).clicked() {
                    self.environment.clock_speed = -self.environment.clock_speed;
                };
                if ui.button(if self.clock_pause { "▶️"[0..3].to_string() } else { "⏸️"[0..3].to_string() } ).clicked() {
                    self.clock_pause = !self.clock_pause;
                };
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.auto_reverse_enabled,"Auto Reverse");
            });
            if let Some(n) = self.sel_link {
                egui::CollapsingHeader::new(format!("Fine tuning (Link {})",n)).default_open(false).show(ui, |ui| {
                    egui::Grid::new("postable3").show(ui, |ui|{
                        ui.label("Time");
                        ui.add(egui::DragValue::new(&mut self.links[n].push_timing).speed(1).clamp_range(0..=200));
                        ui.end_row();
                        ui.label("Span");
                        ui.add(egui::DragValue::new(&mut self.links[n].push_span).speed(1).clamp_range(0..=100));
                        ui.end_row();
                        ui.label("Force");
                        ui.add(egui::DragValue::new(&mut self.links[n].push_strength).speed(0.1));
                        ui.end_row();
                        if ui.button("Remove").clicked() {
                            self.links[n].push_strength = 0.0;
                            self.links[n].push_timing = 0;
                            self.links[n].push_span = 0;
                        };
                    });
                });
            }
        });
        
    });
    ctx.request_repaint()
 }
fn name(&self) -> &str { "botz" }

}
pub fn make_start() -> State {

    let mut s = State {
        auto_reverse_cycle: 0,
        auto_reverse_enabled: true,
        ceiling: 594.0,
        current_phase: 0,
        cycle_time: 0,
        drag_dot: None,
        environment: Environment {
            atmosphere: 0.01,
            clock_speed: 3,
            gravity: 0.4,
            left_wind: 0.0,
            tension: 0.9,
            wall_bounce: 0.4,
            wall_friction: 0.7,
        },
        walls: Walls {
            left: true,
            right: true,
            ceiling: true,
            floor: true,
        },
        height: 600,
        hover_link: None,
        hover_vertex: None,
        links: Vec::new(),
        mode: 0,
        mouse_x: 0.0,
        mouse_y: 0.0,
        rightwall: 797.0,
        sel_link: None,
        sel_vertex: None,
        sub_mode : 0,
        sub_mode_data: 0,
        vertices: Vec::new(),
        width: 800,
        clock_pause: false,
        triangles_updated: false,
        triangles: std::collections::HashSet::new(),
        display_options: DisplayOptions { 
            show_link_handles: false, 
            zoom2x: false, 
            show_links: true, 
            show_vertices: true, 
            show_wheels: true,
            num_wheel_spokes: 6,
            shade_body: true,
            shade_wheels: true,
            background_color: egui::Color32::from_rgba_premultiplied(255, 255, 245, 255),
            shade_color: egui::Color32::from_rgba_premultiplied(0, 86, 116, 45),
            wheel_shade_color: egui::Color32::from_rgba_unmultiplied(0, 255, 255, 64),
            link_color: egui::Color32::from_rgb(0, 0, 255),
            selection_color: egui::Color32::from_rgb(255, 0, 255),
            hover_color: egui::Color32::from_rgb(255, 0, 0),
            vertex_color: egui::Color32::from_rgb(0, 0, 0),
            link_handle_color: egui::Color32::from_rgb(0, 0, 0),
            wheel_color: egui::Color32::from_rgb(0, 0, 0),
            wheel_spoke_color: egui::Color32::from_rgba_premultiplied(70, 156, 150, 255),
        } 
    };
    s.legacy_parse(include_str!("../walker.botz")).unwrap();
    s
}