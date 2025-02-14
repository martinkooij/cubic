use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::LineWriter;
use clap::Parser;

fn convert_toint32(val: impl TryInto<i32>) -> i32 {
    // use try_into trait method
    let val: i32 = match val.try_into() {
        Ok(v) => v,
        Err(_) => panic!("couldn't fit in i32"),
    };
    val
}

fn chop_to_integer(val: f64) -> i32 {
    val as i32
}

fn make_float(val: i32) -> f64 {
    val as f64
}

fn convert_tousize(val: impl TryInto<usize>) -> usize {
    // use try_into trait method
    let val: usize = match val.try_into() {
        Ok(v) => v,
        Err(_) => panic!("couldn't fit in usize"),
    };
    val
}
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// upslope in seconds
    #[arg(short, long, default_value_t = 2.0)]
    upslope: f64,

    /// downslope in seconds
    #[arg(short, long, default_value_t = 2.0)]
    downslope: f64,

    /// filename to write to
    #[arg(short, long, default_value_t = String::from("allpoints-csv.csv"))]
    filename: String,
}

fn main() {
    const NO_DETAILED_POINTS_PER_SEC: i32 = 100;
    const NO_ROUGH_POINTS_PER_SEC: i32 = 1;
    const _MAX_VELOCITY: f64 = 5.0;
    const MAX_ACCELERATION: f64 = 2.0;
    const ACCELERATION_UP: f64 = MAX_ACCELERATION;
    const ACCELERATION_DOWN: f64 = -MAX_ACCELERATION;
    const TOTAL_SEC: i32 = 10;
    const NO_MAIN_POINTS: i32 = TOTAL_SEC * NO_ROUGH_POINTS_PER_SEC;
    const NO_TOTAL_POINTS: i32 = TOTAL_SEC * NO_DETAILED_POINTS_PER_SEC;
    const START_POSITION: f64 = 0.0;
    const START_VELOCITY: f64 = 0.0;

    let args = Args::parse();
    let slope_up_time_sec = args.upslope ;
    let slope_down_time_sec=args.downslope ;
    let flat_time_sec = 10.0 - (slope_up_time_sec + slope_down_time_sec);

    #[derive(Debug, Copy, Clone)]
    struct Movepoint {
        tick: i32,
        position: f64,
        velocity: f64,
        acceleration: f64,
    }

    #[derive(Debug, Copy, Clone)]
    struct Pvtpoint {
        tick: i32,
        position: f64,
        velocity: f64,
    }

    let calculate_position = |tick:i32|  -> f64 {
        if tick <= chop_to_integer(slope_up_time_sec * NO_DETAILED_POINTS_PER_SEC as f64) {
            0.5
                * ACCELERATION_UP
                * (tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64)
                * (tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64)
        } else if tick <= chop_to_integer((slope_up_time_sec +flat_time_sec)  * NO_DETAILED_POINTS_PER_SEC as f64) {
            (0.5
            * ACCELERATION_UP
            * slope_up_time_sec
            * slope_up_time_sec) 
            +
            ((ACCELERATION_UP * slope_up_time_sec)*((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - slope_up_time_sec))
        } else {
            (0.5
                * ACCELERATION_UP
                * slope_up_time_sec
                * slope_up_time_sec) 
            +
            (ACCELERATION_UP * slope_up_time_sec)*flat_time_sec
            +
            (ACCELERATION_UP * slope_up_time_sec)
                 *
            ((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - (slope_up_time_sec + flat_time_sec))
            +
            0.5
                * ACCELERATION_DOWN
                * ((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - (slope_up_time_sec + flat_time_sec))
                * ((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - (slope_up_time_sec + flat_time_sec))
            }
    };

    let calculate_velocity =  |tick: i32| -> f64 {
        if tick <= chop_to_integer(slope_up_time_sec * NO_DETAILED_POINTS_PER_SEC as f64) {
                ACCELERATION_UP
                * 
                (tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64)
        } else if tick <= chop_to_integer((slope_up_time_sec +flat_time_sec)  * NO_DETAILED_POINTS_PER_SEC as f64) {
            ACCELERATION_UP * slope_up_time_sec
        } else {
            ACCELERATION_UP * slope_up_time_sec
            +
            ACCELERATION_DOWN * ((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - (slope_up_time_sec + flat_time_sec))
        }
    };

    let calculate_acceleration = |tick: i32| -> f64 {
        if tick <= chop_to_integer(slope_up_time_sec * NO_DETAILED_POINTS_PER_SEC as f64) {
            ACCELERATION_UP
        } else if tick <= chop_to_integer((slope_up_time_sec +flat_time_sec)  * NO_DETAILED_POINTS_PER_SEC as f64) {
            0.0
        } else {
            ACCELERATION_DOWN 
        }
    };

    fn cubic_inter_fn (x0:Pvtpoint, x1: Pvtpoint) -> impl Fn(i32)->Movepoint {
        let a:f64 = 2.0 * x0.position- 2.0*x1.position + x0.velocity + x1.velocity ;
        let b:f64 = -3.0 * x0.position +3.0 * x1.position - 2.0 * x0.velocity - x1.velocity ;
        let c:f64 = x0.velocity ;
        let d:f64 = x0.position ;
        move |tick: i32| {
        let t = make_float(tick - x0.tick) / make_float(NO_DETAILED_POINTS_PER_SEC);
            Movepoint {
                tick,
                position: a*t*t*t + b*t*t + c*t + d,
                velocity: 3.0*a*t*t + 2.0*b*t + c,
                acceleration: 6.0*a*t + 2.0*b,
            }
        }
    }

    let mut pvtpoints: [Pvtpoint; NO_MAIN_POINTS as usize + 1] = [Pvtpoint {
        position: START_POSITION,
        velocity: START_VELOCITY,
        tick: 0,
    }; NO_MAIN_POINTS as usize + 1];

    let mut allpoints: [Movepoint; NO_TOTAL_POINTS as usize + 1] = [Movepoint {
        tick: 0,
        position: START_POSITION,
        velocity: START_VELOCITY,
        acceleration: ACCELERATION_UP,
    }; NO_TOTAL_POINTS as usize + 1];

    let mut cubicpoints: [Movepoint; NO_TOTAL_POINTS as usize + 1] = [Movepoint {
        tick: 0,
        position: START_POSITION,
        velocity: START_VELOCITY,
        acceleration: ACCELERATION_UP,
    }; NO_TOTAL_POINTS as usize + 1];

    // set up files
    let file = File::create(args.filename).expect("Unable to create file");
    let mut file = LineWriter::new(file);

    //Ideal Situation
    
    
    for i in 0..NO_MAIN_POINTS+1 {
        pvtpoints[i as usize].tick = i*(NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC);
        pvtpoints[i as usize].position = calculate_position(i*(NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC));
        pvtpoints[i as usize].velocity = calculate_velocity(i*(NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC));
    }

    for i in 0..NO_TOTAL_POINTS+1 {
        allpoints[i as usize].tick = i;
        allpoints[i as usize].position = calculate_position(i);
        allpoints[i as usize].velocity = calculate_velocity(i);
        allpoints[i as usize].acceleration = calculate_acceleration(i);
    }

    // fill cubic interpolations
    let mut interpol = cubic_inter_fn(pvtpoints[0],pvtpoints[1]);
    cubicpoints[0 as usize] = interpol(0);
    for i in 1..NO_TOTAL_POINTS {
        if i % (NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC) == 0 {
            interpol = cubic_inter_fn(pvtpoints[convert_tousize( i / (NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC))],
            pvtpoints[convert_tousize((i / (NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC))+1)]);
        }
        cubicpoints[i as usize] = interpol(i) 
    }
    cubicpoints[NO_TOTAL_POINTS as usize]=interpol(NO_TOTAL_POINTS);

    
    writeln!(file, "tick,position,velocity,acceleration,cubic_p,cubic_v, cubic_a,dpos,dvel").expect("write failed");

    for i in 0..NO_TOTAL_POINTS+1 {
        if i % (NO_DETAILED_POINTS_PER_SEC / NO_ROUGH_POINTS_PER_SEC) != 0 {
        writeln!(
            file,
            "{},{},{},{},{},{},{}",
            allpoints[i as usize].tick,
            allpoints[i as usize].position,
            allpoints[i as usize].velocity,
            allpoints[i as usize].acceleration,
            cubicpoints[i as usize].position,
            cubicpoints[i as usize].velocity,
            cubicpoints[i as usize].acceleration,
        )
        .expect("write points failed");
        } else {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{}",
                allpoints[i as usize].tick,
                allpoints[i as usize].position,
                allpoints[i as usize].velocity,
                allpoints[i as usize].acceleration,
                cubicpoints[i as usize].position,
                cubicpoints[i as usize].velocity,
                cubicpoints[i as usize].acceleration,
                pvtpoints[convert_tousize(i / (NO_DETAILED_POINTS_PER_SEC / NO_ROUGH_POINTS_PER_SEC))].position,
                pvtpoints[convert_tousize(i /(NO_DETAILED_POINTS_PER_SEC / NO_ROUGH_POINTS_PER_SEC))].velocity,
            )
         .expect("write points failed");
        }
    }
}
