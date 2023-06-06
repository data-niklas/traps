use super::lib::{Gesture, GestureAttributes, GestureRecorder, Point};
use super::ui;

fn parse_hex(hex_code: &str) -> u32 {
    let r: u8 = u8::from_str_radix(&hex_code[1..3], 16).unwrap();
    let g: u8 = u8::from_str_radix(&hex_code[3..5], 16).unwrap();
    let b: u8 = u8::from_str_radix(&hex_code[5..7], 16).unwrap();
    let a: u8 = u8::from_str_radix(&hex_code[7..9], 16).unwrap();

    ui::color_to_argb(r as u32, g as u32, b as u32, a as u32)
}

pub struct Config {
    pub fg: u32,
    pub bg: u32,
    pub r: u32,
    pub gestures: Vec<Gesture>,
}

impl Config {
    pub fn new() -> Config {
        let content = Config::config_file_content();

        let mut fg = ui::color_to_argb(255, 255, 255, 255);
        let mut bg = ui::color_to_argb(0, 0, 0, 150);
        let mut r = 10;
        let mut gestures = Vec::new();

        let mut attributes = GestureAttributes::default();

        let relevant_lines = content.lines().filter(|line| !line.starts_with('#'));
        for line in relevant_lines {
            let vec: Vec<&str> = line.splitn(2, '=').collect();
            if vec.len() == 2 {
                let key = vec.get(0).unwrap().trim();
                let value = vec.get(1).unwrap().trim();

                match key {
                    "name" => {
                        attributes.name = value;
                    }
                    "is_relative" => {
                        attributes.is_relative = Self::parse_is_relative(value);
                    }
                    "action" => {
                        attributes.action = value;
                    }
                    "tolerance" => {
                        attributes.tolerance = Self::parse_tolerance(value);
                    }
                    "points" => {
                        let points = Self::parse_points(value);
                        let mut gesture = Gesture::new(&attributes);
                        gesture.add_points(points);
                        gestures.push(gesture);
                        attributes = GestureAttributes::default();
                    }
                    "bg" => {
                        bg = parse_hex(value);
                    }
                    "fg" => {
                        fg = parse_hex(value);
                    }
                    "r" => {
                        r = value.parse().expect("Expected int in config");
                    }
                    _ => {}
                }
            }
        }

        Config {
            fg,
            bg,
            r,
            gestures,
        }
    }

    fn parse_is_relative(value: &str) -> bool{
        match value.parse(){
            Err(_) => false,
            Ok(value) => value
        }
    }

    fn parse_tolerance(value: &str) -> f32{
        match value.parse(){
            Err(_) => GestureRecorder::DEFAULT_TOLERANCE,
            Ok(value) => value
        }
    }

    fn parse_single_coordinate(text: &str) -> i16{
        match text.parse(){
            Err(_) => 0,
            Ok(value) => value
        }
    }

    fn parse_default_point(brackets: &str) -> Option<(i16, i16)> {
        let inside = brackets[1..brackets.len() - 1].trim();
            let xy: Vec<&str> = inside.split(' ').collect();
            if xy.len() == 2 {
                return Some((
                    Self::parse_single_coordinate(xy.get(0).unwrap()),
                    Self::parse_single_coordinate(xy.get(1).unwrap()),
                ));
            }
        None
    }

    fn parse_point(points: &mut Vec<Point>, brackets: &str){
        if brackets.starts_with('(') && brackets.ends_with(')') {
            if let Some((x, y)) = Self::parse_default_point(brackets){
                points.push(Point::new(x,y));
            }
        }
        else if brackets.starts_with("Circle(") && brackets.ends_with(')'){

        }
    }

    fn parse_points(value: &str) -> Vec<Point> {
        let mut points = Vec::new();
        for p in value.split(',') {
            Self::parse_point(&mut points, p.trim());
        }
        points
    }

    fn config_file_content() -> String {
        let mut configdir = dirs::config_dir().expect("Did not find config dir");
        configdir.push("traps");
        if !configdir.exists() {
            std::fs::create_dir(&configdir);
        }
        configdir.push("trapsrc");
        match configdir.exists() {
            true => std::fs::read_to_string(configdir).expect("Could not read file"),
            false => "".to_owned(),
        }
    }
}
