use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub fn new(x: i16, y: i16) -> Point {
        Point { x, y }
    }

    pub fn matched(
        p1: &Point,
        p2: &Point,
        p3: &Point,
        p4: &Point,
        xscale: f32,
        yscale: f32,
        tolerance: f32,
    ) -> bool {
        let xdif1 = p2.x - p1.x;
        let ydif1 = p2.y - p1.y;
        let xdif2 = p4.x - p3.x;
        let ydif2 = p4.y - p3.y;

        (xdif1 as f32 * xscale - xdif2 as f32).abs() < tolerance
            && (ydif1 as f32 * yscale - ydif2 as f32).abs() < tolerance
    }

    pub fn bigger(&self, p1: &Point) -> bool {
        self.x > p1.x && self.y > p1.y
    }

    pub fn smaller(&self, p1: &Point) -> bool {
        self.x < p1.x && self.y < p1.y
    }

    pub fn between(&self, p1: &Point, p2: &Point) -> bool {
        self.smaller(p2) && self.bigger(p1)
    }

    pub fn bounds(points: &Vec<Point>) -> (i16, i16) {
        let mut smallx = i16::max_value();
        let mut smally = i16::max_value();
        let mut bigx = i16::min_value();
        let mut bigy = i16::min_value();
        for point in points {
            if point.x > bigx {
                bigx = point.x;
            } else if point.x < smallx {
                smallx = point.x;
            }
            if point.y > bigy {
                bigy = point.y;
            } else if point.y < smally {
                smally = point.y;
            }
        }

        (bigx - smallx, bigy - smally)
    }
}

pub struct GestureRecorder {
    pub is_tracking: bool,
    fast_trigger: bool,
    points: Vec<Point>,
    gestures: Vec<Arc<Gesture>>,
    tracked_gestures: Vec<TrackedGesture>,
    listener: Box<dyn Fn(Arc<Gesture>) -> bool>,
}

//Constraints are checked at the end
//Fast_trigger only works with non-relative gestures
impl GestureRecorder {
    pub fn new(listener: Box<dyn Fn(Arc<Gesture>) -> bool>) -> GestureRecorder {
        GestureRecorder {
            is_tracking: false,
            fast_trigger: false,
            points: Vec::new(),
            gestures: Vec::new(),
            tracked_gestures: Vec::new(),
            listener,
        }
    }
    pub const DEFAULT_TOLERANCE: f32 = 20.0;

    pub fn set_fast_trigger(&mut self, fast_trigger: bool) {
        if !self.is_tracking {
            self.fast_trigger = fast_trigger;
        }
    }

    pub fn fast_trigger_activated(&self) -> bool {
        self.fast_trigger
    }

    pub fn register_gesture(&mut self, mut gesture: Gesture) {
        gesture.calculate_bounds();
        self.gestures.push(Arc::new(gesture));
    }

    pub fn start(&mut self) {
        self.points.clear();
        for gesture in &self.gestures {
            self.tracked_gestures
                .push(TrackedGesture::new(gesture.clone()));
        }

        self.is_tracking = true;
    }

    pub fn stop(&mut self) {
        if !self.fast_trigger && !self.points.is_empty() {
            let mut found_gesture: Arc<Option<Arc<Gesture>>> = Arc::new(None);
            let (pwidth, pheight) = Point::bounds(&self.points);

            let pfirst = self.points.first().unwrap();
            self.tracked_gestures = self
                .tracked_gestures
                .into_iter()
                .map(|mut tracked_gesture| {
                    {
                        if tracked_gesture.is_relative() {
                            tracked_gesture.determine_scale(pwidth, pheight);
                        }
                    }

                    let (gesture_matched, tracked_gesture) =
                        self.check_likeliest_match(tracked_gesture, pfirst);
                    if gesture_matched {
                        if let Some(found_gesture_inner) = *found_gesture {
                            if tracked_gesture.points_count() > found_gesture_inner.points_count() {
                                found_gesture = Arc::new(Some(tracked_gesture.gesture.clone()));
                            } else if tracked_gesture.points_count()
                                == found_gesture_inner.points_count()
                                && !tracked_gesture.is_relative()
                            {
                                found_gesture = Arc::new(Some(tracked_gesture.gesture.clone()));
                            }
                        } else {
                            found_gesture = Arc::new(Some(tracked_gesture.gesture.clone()));
                        }
                    }

                    tracked_gesture
                })
                .collect();

            if let Some(found_gesture_inner) = &*found_gesture {
                self.notify(found_gesture_inner.clone());
            }
        }
        self.is_tracking = false;
    }

    pub fn track(&mut self, plast: Point) {
        /*if self.fast_trigger {
            let mut i = 0;
            while i < self.gestures.len() {
                let mut tracked_gesture = self.tracked_gestures.get_mut(i).unwrap();
                let gesture = &tracked_gesture.gesture;
                let glast = gesture.get_point(tracked_gesture.matched_points);

                if self.points.is_empty() {
                    tracked_gesture.inc_matched();
                } else {
                    let pfirst = self.points.first().unwrap();
                    let gfirst = gesture.first();
                    if Point::matched(pfirst, &plast, gfirst, glast, 1.0, 1.0, gesture.tolerance) {
                        tracked_gesture.inc_matched();
                    }
                }
                if tracked_gesture.all_matched() {
                    if gesture.constraints_matching(&self.points) {
                        if self.notify(&tracked_gesture.gesture) {
                            self.stop();
                            break;
                        } else {
                            self.tracked_gestures.remove(i);
                        }
                    } else {
                        self.tracked_gestures.remove(i);
                    }
                } else {
                    i += 1;
                }
            }
        }
        self.points.push(plast);*/
    }

    fn check_likeliest_match(
        &self,
        tracked_gesture: TrackedGesture,
        pfirst: &Point,
    ) -> (bool, TrackedGesture) {
        let gfirst = tracked_gesture.first();
        for point in &self.points {
            let glast = tracked_gesture.get_point(tracked_gesture.matched_points);
            if Point::matched(
                pfirst,
                &point,
                gfirst,
                glast,
                tracked_gesture.xscale,
                tracked_gesture.yscale,
                tracked_gesture.tolerance(),
            ) {
                tracked_gesture.inc_matched();
            }
            if tracked_gesture.all_matched() {
                return (
                    tracked_gesture.constraints_matching(&self.points),
                    tracked_gesture,
                );
            }
        }
        (false, tracked_gesture)
    }

    pub fn notify(&self, gesture: Arc<Gesture>) -> bool {
        (*self.listener)(gesture)
    }
}

pub struct GestureAttributes<'a> {
    pub name: &'a str,
    pub action: &'a str,
    pub is_relative: bool,
    pub tolerance: f32,
}

impl<'a> GestureAttributes<'a> {
    pub fn default() -> GestureAttributes<'a> {
        GestureAttributes {
            name: "",
            action: "",
            is_relative: false,
            tolerance: GestureRecorder::DEFAULT_TOLERANCE,
        }
    }
}

//Points should start at (0,0)
//Positioning is possible through constraints
#[derive(Debug, Clone)]
pub struct Gesture {
    points: Vec<Point>,
    pub is_relative: bool,
    constraints: Vec<Constraint>,
    pub tolerance: f32,
    pub name: String,
    pub action: String,
    width: i16,
    height: i16,
}

impl Gesture {
    pub fn new(attributes: &GestureAttributes) -> Gesture {
        Gesture {
            is_relative: attributes.is_relative,
            name: attributes.name.to_owned(),
            action: attributes.action.to_owned(),
            points: Vec::new(),
            tolerance: attributes.tolerance,
            constraints: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn calculate_bounds(&mut self) {
        let (width, height) = Point::bounds(&self.points);
        self.width = width;
        self.height = height;
    }

    pub fn add_points(&mut self, mut points: Vec<Point>) {
        self.points.append(&mut points);
    }

    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    pub fn get_point(&self, index: usize) -> &Point {
        self.points.get(index).unwrap()
    }

    pub fn constraints_matching(&self, points: &Vec<Point>) -> bool {
        let first = points.first().unwrap();
        let last = points.last().unwrap();
        for constraint in &self.constraints {
            let p;
            let area;
            match constraint {
                Constraint::StartArea(a) => {
                    p = first;
                    area = a;
                }
                Constraint::StopArea(a) => {
                    p = last;
                    area = a;
                }
            }
            match area {
                Area::Between(start, end) => {
                    if !p.between(start, end) {
                        return false;
                    }
                }
                Area::Smaller(max) => {
                    if !p.smaller(max) {
                        return false;
                    }
                }
                Area::Bigger(min) => {
                    if !p.bigger(min) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn first(&self) -> &Point {
        self.points.first().unwrap()
    }

    pub fn last(&self) -> &Point {
        self.points.last().unwrap()
    }

    pub fn points_count(&self) -> usize {
        self.points.len()
    }
}

#[derive(Debug, Clone)]
pub struct TrackedGesture {
    gesture: Arc<Gesture>,
    pub matched_points: usize,
    pub xscale: f32,
    pub yscale: f32,
}

impl TrackedGesture {
    pub fn new(gesture: Arc<Gesture>) -> TrackedGesture {
        TrackedGesture {
            gesture,
            matched_points: 0,
            xscale: 1.0,
            yscale: 1.0,
        }
    }

    pub fn inc_matched(&mut self) {
        self.matched_points += 1;
    }

    pub fn all_matched(&self) -> bool {
        self.matched_points == self.gesture.points.len()
    }

    pub fn determine_scale(&mut self, pwidth: i16, pheight: i16) {
        self.xscale = self.gesture.width as f32 / pwidth as f32;
        self.yscale = self.gesture.height as f32 / pheight as f32;
    }

    pub fn is_relative(&self) -> bool {
        self.gesture.is_relative
    }

    pub fn first(&self) -> &Point {
        self.gesture.points.first().unwrap()
    }

    pub fn last(&self) -> &Point {
        self.gesture.points.last().unwrap()
    }

    pub fn points_count(&self) -> usize {
        self.gesture.points.len()
    }

    pub fn gesture(&self) -> &Gesture {
        &self.gesture
    }

    pub fn get_point(&self, index: usize) -> &Point {
        self.gesture.get_point(index)
    }

    pub fn constraints_matching(&self, points: &Vec<Point>) -> bool {
        self.gesture.constraints_matching(points)
    }

    pub fn tolerance(&self) -> f32 {
        self.gesture.tolerance
    }
}

#[derive(Debug, Clone)]
pub enum Constraint {
    StartArea(Area),
    StopArea(Area),
}

#[derive(Debug, Clone)]
pub enum Area {
    Smaller(Point),
    Bigger(Point),
    Between(Point, Point),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first() {
        let mut recorder = GestureRecorder::new(Box::new(|gesture| {
            println!("Gesture was found {:?}", gesture);
            true
        }));
        let mut attributes = GestureAttributes::default();
        attributes.name = "Right-swipe";
        attributes.action = "some action";
        attributes.is_relative = true;
        let mut gesture = Gesture::new(&attributes);
        gesture.add_points(vec![Point::new(0, 0), Point::new(-100, 0)]);
        recorder.register_gesture(gesture);

        recorder.start();
        recorder.track(Point::new(300, 422));
        recorder.track(Point::new(500, 488));
        recorder.stop();
    }
}
