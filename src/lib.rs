use snowflake::ProcessUniqueId;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Point{
    pub x: i32,
    pub y: i32
}

impl Point{
    pub fn new(x: i32, y: i32) -> Point{
        Point{
            x:x,
            y:y
        }
    }

    pub fn distance_to(&self, point: &Point) -> f32{
        let xdif = point.x - self.x;
        let ydif = point.y - self.y;
        let c: f32 = (xdif*xdif + ydif*ydif) as f32 ;
        return c.sqrt().abs();
    }

    //Relative to 0,0
    pub fn hypotenuse(&self) -> f32{
        let c: f32 = (self.x*self.x + self.y*self.y) as f32;
        return c.sqrt().abs();
    }

    pub fn matched(p1: &Point, p2: &Point, p3: &Point, p4: &Point, xscale: f32, yscale: f32, tolerance: f32) -> bool{
        let xdif1 = p2.x - p1.x;
        let ydif1 = p2.y - p1.y;
        let xdif2 = p4.x - p3.x;
        let ydif2 = p4.y - p3.y;
        
        return (xdif1 as f32*xscale - xdif2 as f32).abs() < tolerance && (ydif1 as f32*yscale - ydif2 as f32).abs() < tolerance;
    }
}





pub struct GestureRecorder{
    pub is_tracking: bool,
    fast_trigger: bool,
    pub tolerance: f32,
    points: Vec<Point>,
    gestures: HashMap<ProcessUniqueId, Gesture>,
    tracked_gestures: Vec<TrackedGesture>,
    listener: Box<dyn Fn(&Gesture) -> bool>
}


//Constraints are checked at the end
//Fast_trigger only works with non-relative gestures
impl GestureRecorder{
    pub fn new(listener: Box<dyn Fn(&Gesture) -> bool>) -> GestureRecorder{
        GestureRecorder{
            is_tracking: false,
            fast_trigger: false,
            tolerance: GestureRecorder::DEFAULT_TOLERANCE,
            points: Vec::new(),
            gestures: HashMap::new(),
            tracked_gestures: Vec::new(),
            listener: listener
        }
    }
    pub const DEFAULT_TOLERANCE: f32 = 100.0;

    pub fn set_fast_trigger(&mut self, fast_trigger: bool){
        if !self.is_tracking{
            self.fast_trigger = fast_trigger;
        }
        else{
            panic!("Fast trigger may only be set, while the GestureRecorder is not tracking points");
        }
    }

    pub fn fast_trigger_activated(&self) -> bool{
        return self.fast_trigger;
    }

    pub fn register_gesture(&mut self, gesture: Gesture){
        self.gestures.insert(gesture.id, gesture);
    }

    pub fn unregister_gesture(&mut self, gesture: Gesture){
        self.gestures.remove(&gesture.id);
    }

    pub fn start(&mut self){
        self.points.clear();
        self.tracked_gestures.clear();

        if self.fast_trigger{
            for (id, gesture) in &self.gestures{
                if !gesture.is_relative{
                    self.tracked_gestures.push(TrackedGesture::new(*id));
                }
            }
        }

        self.is_tracking = true;
    }

    pub fn stop(&mut self){
        if !self.fast_trigger && self.points.len() > 0{

            let mut found_gesture: Option<&Gesture> = None;

            let pfirst = self.points.first().unwrap();
            let plast = self.points.last().unwrap();

            for (id, gesture) in &self.gestures{
                let mut tracked_gesture = TrackedGesture::new(*id);
                let gfirst = gesture.first();
                let glast = gesture.last();

                if gesture.is_relative{
                    tracked_gesture.determine_scale(pfirst, plast, gfirst, glast);
                }

                if let Some(returned_gesture) = self.check_likeliest_match(&mut tracked_gesture, gesture, pfirst){
                    if let Some(found_gesture_inner) = found_gesture{
                        if gesture.points_count() > found_gesture_inner.points_count(){
                            found_gesture = Some(returned_gesture);
                        }
                    } 
                    else{
                        found_gesture = Some(returned_gesture);
                    }
                    found_gesture = Some(returned_gesture);
                }
                
            }

            if let Some(found_gesture_inner) = found_gesture{
                self.notify(found_gesture_inner);
            }

        }
        self.is_tracking = false;
    }

    pub fn track(&mut self, plast: Point){
        if self.fast_trigger{
            let mut i = 0;
            while i < self.tracked_gestures.len(){
                let tracked_gesture = self.tracked_gestures.get_mut(i).unwrap();
                let gesture = self.gestures.get(&tracked_gesture.id).unwrap();
                let glast = gesture.get_point(tracked_gesture.matched_points);

                if self.points.len() == 0{
                    tracked_gesture.inc_matched();
                }
                else{
                    let pfirst = self.points.first().unwrap();
                    let gfirst = gesture.first();
                    if Point::matched(pfirst, &plast, gfirst, glast, 1.0, 1.0, self.tolerance){
                        tracked_gesture.inc_matched();
                    }
                }
                if tracked_gesture.all_matched(gesture){
                    if gesture.constraints_matching(&self.points){
                        if self.notify(gesture){
                            self.stop();
                            break;
                        }
                        else{
                            self.tracked_gestures.remove(i);
                        }
                    }
                    else{
                        self.tracked_gestures.remove(i);
                    }
                }
                else{
                    i+=1;
                }
            }
        }
        self.points.push(plast);
    }

    fn check_likeliest_match<'a>(&self, tracked_gesture: &mut TrackedGesture, gesture: &'a Gesture, pfirst: &Point) -> Option<&'a Gesture>{
        let gfirst = gesture.first();
        for point in &self.points{
            let glast = gesture.get_point(tracked_gesture.matched_points);
            if Point::matched(pfirst, &point, gfirst, glast, tracked_gesture.xscale, tracked_gesture.yscale, self.tolerance){
                tracked_gesture.inc_matched();
            }
            if tracked_gesture.all_matched(gesture){
                if gesture.constraints_matching(&self.points){

                    return Some(gesture);
                }
                else{
                    return None;
                }
            }
        }
        return None;
    }

    pub fn notify(&self, gesture: &Gesture) -> bool{
        return (*self.listener)(gesture);
    }


}


//Points should start at (0,0)
//Positioning is possible through constraints
#[derive(Debug)]
pub struct Gesture{
    points: Vec<Point>,
    pub is_relative: bool,
    constraints: Vec<Constraint>,
    pub name: String,
    pub id: ProcessUniqueId
}

impl Gesture{
    pub fn new(name: String, is_relative: bool) -> Gesture{
        Gesture{
            is_relative: is_relative,
            name: name,
            points: Vec::new(),
            constraints: Vec::new(),
            id: ProcessUniqueId::new()
        }
    }

    pub fn add_points(&mut self, mut points: Vec<Point>){
        self.points.append(&mut points);
    }

    pub fn add_point(&mut self, point: Point){
        self.points.push(point);
    }

    pub fn get_point(&self, index: usize) -> &Point{
        return self.points.get(index).unwrap();
    }

    pub fn constraints_matching(&self, points: &Vec<Point>) -> bool{
        return true;
    }

    pub fn first(&self) -> &Point{
        return self.points.first().unwrap();
    }

    pub fn last(&self) -> &Point{
        return self.points.last().unwrap();
    }

    pub fn points_count(&self) -> usize{
        return self.points.len();
    }
}


#[derive(Debug)]
pub struct TrackedGesture{
    pub id: ProcessUniqueId,
    pub matched_points: usize,
    pub xscale: f32,
    pub yscale: f32
}

impl TrackedGesture{
    pub fn new(id: ProcessUniqueId) -> TrackedGesture{
        TrackedGesture{
            id: id,
            matched_points: 0,
            xscale: 1.0,
            yscale: 1.0
        }
    }

    pub fn inc_matched(&mut self){
        self.matched_points+=1;
    }

    pub fn all_matched(&self, gesture: &Gesture) -> bool{
        return self.matched_points == gesture.points.len();
    }

    pub fn determine_scale(&mut self, p1: &Point, p2: &Point, p3: &Point, p4: &Point){
        let xdif1 = p2.x - p1.x;
        let ydif1 = p2.y - p1.y;
        let xdif2 = p4.x - p3.x;
        let ydif2 = p4.y - p3.y;
        self.xscale = (xdif2 as f32 / xdif1 as f32).abs();
        self.yscale = (ydif2 as f32 / ydif1 as f32).abs();
    }
}

#[derive(Debug)]
pub struct Constraint{

}


#[cfg(test)]
mod tests {

}