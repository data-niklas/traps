use super::*;

use xcb::ffi::*;
use xcb::*;

pub struct UI {
    window: u32,
    conn: Connection,
    gc: u32,
    fg: u32,
    bg: u32,
    radius: u32,
    screen_num: i32,
    width: u16,
    height: u16,
    visible: bool,
}

impl UI {
    pub fn new(config: &config::Config) -> UI {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
        UI {
            fg: config.fg,
            bg: config.bg,
            radius: config.r,
            gc: conn.generate_id(),
            window: conn.generate_id(),
            conn,
            screen_num,
            visible: false,
            width: 0,
            height: 0,
        }
    }

    pub fn init(&mut self) {
        self.create_window();
        self.make_gc();
    }

    fn window_values(&self, colormap: u32) -> [(u32, u32); 5] {
        [
            (xcb::CW_BACK_PIXEL, self.bg),
            (xcb::CW_BORDER_PIXEL, self.bg),
            (xcb::CW_OVERRIDE_REDIRECT, 1),
            (
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_EXPOSURE
                    | xcb::EVENT_MASK_KEY_PRESS
                    | xcb::EVENT_MASK_BUTTON_1_MOTION
                    | xcb::EVENT_MASK_BUTTON_PRESS
                    | xcb::EVENT_MASK_BUTTON_RELEASE,
            ),
            (xcb::CW_COLORMAP, colormap),
        ]
    }

    fn create_window(&mut self) {
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(self.screen_num as usize).unwrap();
        self.width = screen.width_in_pixels();
        self.height = screen.height_in_pixels();
        let colormap = self.conn.generate_id();
        unsafe {
            let visual = get_visual(screen.ptr).expect("Your screen does not support argb");
            create_colormap(
                &self.conn,
                XCB_COLORMAP_ALLOC_NONE as u8,
                colormap,
                screen.root(),
                visual.as_ref().unwrap().visual_id,
            );
            let values = self.window_values(colormap);
            xcb::create_window(
                &self.conn,
                32,
                self.window,
                screen.root(),
                0,
                0,
                self.width,
                self.height,
                0,
                xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                visual.as_ref().unwrap().visual_id,
                &values,
            );
        }

        xcb::change_property(
            &self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.window,
            xcb::ATOM_WM_NAME,
            xcb::ATOM_STRING,
            8,
            constants::APPNAME.as_bytes(),
        );
        self.conn.flush();
    }

    fn make_gc(&self) {
        xcb::create_gc(
            &self.conn,
            self.gc,
            self.window,
            &[
                (xcb::GC_FUNCTION, xcb::xproto::GX_COPY),
                (xcb::GC_FOREGROUND, self.fg),
                (xcb::GC_BACKGROUND, self.bg),
                (xcb::GC_LINE_WIDTH, self.radius * 2),
                (xcb::GC_GRAPHICS_EXPOSURES, 1),
            ],
        );
    }

    fn draw_point(&self, x: i16, y: i16){
        xcb::poly_fill_arc(
            &self.conn,
            self.window,
            self.gc,
            &[xcb::Arc::new(
                x - self.radius as i16,
                y - self.radius as i16,
                self.radius as u16 * 2,
                self.radius as u16 * 2,
                0,
                360 << 6,
            )],
        );
        //xcb::poly_line(&self.conn, 0, self.window, self.gc, &[xcb::Point::new(oldx, oldy), xcb::Point::new(motion.event_x(), motion.event_y())]);
        self.conn.flush();
    }

    fn clear(&self) {
        xcb::change_gc(&self.conn, self.gc, &[(xcb::GC_FOREGROUND, self.bg)]);
        xcb::poly_fill_rectangle(
            &self.conn,
            self.window,
            self.gc,
            &[xcb::Rectangle::new(0, 0, self.width, self.height)],
        );
        xcb::change_gc(&self.conn, self.gc, &[(xcb::GC_FOREGROUND, self.fg)]);
        self.conn.flush();
    }

    pub fn set_visible(&mut self, visible: bool) {
        if visible {
            xcb::map_window(&self.conn, self.window);
            xcb::set_input_focus(&self.conn, XCB_INPUT_FOCUS_PARENT as u8, self.window, 0);
        } else {
            xcb::unmap_window(&self.conn, self.window);
        }
        self.conn.flush();
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn event_loop(&mut self, listener: Box<dyn Fn(Event)>) {
        loop {
            let event = self.conn.wait_for_event();
            match event {
                None => {
                    break;
                }
                Some(event) => {
                    let r = event.response_type() & !0x80;
                    match r {
                        xcb::EXPOSE => {}
                        xcb::KEY_PRESS => {
                            let key_press: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                            if key_press.detail() == 9 {
                                break;
                            } else if key_press.detail() == 23 {
                                self.set_visible(false);
                            } else {
                                println!("{}", key_press.detail());
                            }
                        }
                        xcb::BUTTON_PRESS => {
                            listener(Event::Start);
                        }
                        xcb::BUTTON_RELEASE => {
                            self.set_visible(false);
                            listener(Event::Stop);
                            self.clear();
                            self.set_visible(true);
                        }
                        xcb::MOTION_NOTIFY => {
                            let motion: &xcb::MotionNotifyEvent =
                                unsafe { xcb::cast_event(&event) };
                            self.draw_point(motion.event_x(), motion.event_y());
                            listener(Event::Point(motion.event_x(), motion.event_y()));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

unsafe fn get_visual(
    screen: *mut xcb::ffi::xcb_screen_t,
) -> Option<*mut xcb::ffi::xcb_visualtype_t> {
    for visual in xcb_screen_allowed_depths_iterator(screen) {
        if visual.depth() == 32 {
            return Some(xcb_depth_visuals(visual.ptr));
        }
    }
    None
}

pub fn color_to_argb(r: u32, g: u32, b: u32, a: u32) -> u32 {
    (a << 24) | ((r << 16) * a / 255) | ((g << 8) * a / 255) | (b * a / 255)
}

#[derive(Debug)]
pub enum Event {
    Point(i16, i16),
    Start,
    Stop
}
