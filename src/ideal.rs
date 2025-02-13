use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::LineWriter;

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

fn convert_tousize(val: impl TryInto<usize>) -> usize {
    // use try_into trait method
    let val: usize = match val.try_into() {
        Ok(v) => v,
        Err(_) => panic!("couldn't fit in usize"),
    };
    val
}

fn main() {
    const NO_DETAILED_POINTS_PER_SEC: i32 = 40;
    const NO_ROUGH_POINTS_PER_SEC: i32 = 1;
    const _MAX_VELOCITY: f64 = 5.0;
    const MAX_ACCELERATION: f64 = 2.0;
    const SLOPE_UP_TIME_SEC: f64 = 2.5;
    const ACCELERATION_UP: f64 = MAX_ACCELERATION;
    const FLAT_TIME_SEC: f64 = 5.0;
    const ACCELERATION_DOWN: f64 = -MAX_ACCELERATION;
    const _SLOPE_DOWN_TIME_SEC: f64 = 2.5;
    const TOTAL_SEC: i32 = 10;
    const NO_MAIN_POINTS: i32 = TOTAL_SEC * NO_ROUGH_POINTS_PER_SEC;
    const NO_TOTAL_POINTS: i32 = TOTAL_SEC * NO_DETAILED_POINTS_PER_SEC;
    const START_POSITION: f64 = 0.0;
    const START_VELOCITY: f64 = 0.0;

    #[derive(Debug, Copy, Clone)]
    struct Pvtpoint {
        tick: i32,
        position: f64,
        velocity: f64,
        acceleration: f64,
    }

    fn calculate_position(tick: i32) -> f64 {
        if tick <= chop_to_integer(SLOPE_UP_TIME_SEC * NO_DETAILED_POINTS_PER_SEC as f64) {
            0.5
                * ACCELERATION_UP
                * (tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64)
                * (tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64)
        } else if tick <= chop_to_integer((SLOPE_UP_TIME_SEC +FLAT_TIME_SEC)  * NO_DETAILED_POINTS_PER_SEC as f64) {
            (0.5
            * ACCELERATION_UP
            * SLOPE_UP_TIME_SEC
            * SLOPE_UP_TIME_SEC) 
            +
            ((ACCELERATION_UP * SLOPE_UP_TIME_SEC)*((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - SLOPE_UP_TIME_SEC))
        } else {
            (0.5
                * ACCELERATION_UP
                * SLOPE_UP_TIME_SEC
                * SLOPE_UP_TIME_SEC) 
            +
            (ACCELERATION_UP * SLOPE_UP_TIME_SEC)*FLAT_TIME_SEC
            +
            (ACCELERATION_UP * SLOPE_UP_TIME_SEC)
                 *
            ((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - (SLOPE_UP_TIME_SEC + FLAT_TIME_SEC))
            +
            0.5
                * ACCELERATION_DOWN
                * ((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - (SLOPE_UP_TIME_SEC + FLAT_TIME_SEC))
                * ((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - (SLOPE_UP_TIME_SEC + FLAT_TIME_SEC))
            }
    }

    fn calculate_velocity (tick: i32) -> f64 {
        if tick <= chop_to_integer(SLOPE_UP_TIME_SEC * NO_DETAILED_POINTS_PER_SEC as f64) {
                ACCELERATION_UP
                * 
                (tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64)
        } else if tick <= chop_to_integer((SLOPE_UP_TIME_SEC +FLAT_TIME_SEC)  * NO_DETAILED_POINTS_PER_SEC as f64) {
            ACCELERATION_UP * SLOPE_UP_TIME_SEC
        } else {
            ACCELERATION_UP * SLOPE_UP_TIME_SEC
            +
            ACCELERATION_DOWN * ((tick as f64 / NO_DETAILED_POINTS_PER_SEC as f64) - (SLOPE_UP_TIME_SEC + FLAT_TIME_SEC))
        }
    }

    fn calculate_acceleration (tick: i32) -> f64 {
        if tick <= chop_to_integer(SLOPE_UP_TIME_SEC * NO_DETAILED_POINTS_PER_SEC as f64) {
            ACCELERATION_UP
        } else if tick <= chop_to_integer((SLOPE_UP_TIME_SEC +FLAT_TIME_SEC)  * NO_DETAILED_POINTS_PER_SEC as f64) {
            0.0
        } else {
            ACCELERATION_DOWN 
        }
    }

    let mut pvtpoints: [Pvtpoint; NO_MAIN_POINTS as usize + 1] = [Pvtpoint {
        tick: 0,
        position: START_POSITION,
        velocity: START_VELOCITY,
        acceleration: ACCELERATION_UP,
    }; NO_MAIN_POINTS as usize + 1];

    let mut allpoints: [Pvtpoint; NO_TOTAL_POINTS as usize + 1] = [Pvtpoint {
        tick: 0,
        position: START_POSITION,
        velocity: START_VELOCITY,
        acceleration: ACCELERATION_UP,
    }; NO_TOTAL_POINTS as usize + 1];

    // set up files
    let file = File::create("trials.csv").expect("Unable to create file");
    let mut file = LineWriter::new(file);

    //Ideal Situation
    
    
    for i in 0..NO_MAIN_POINTS+1 {
        pvtpoints[i as usize].tick = convert_toint32(i*(NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC));
        pvtpoints[i as usize].position = calculate_position(i*(NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC));
        pvtpoints[i as usize].velocity = calculate_velocity(i*(NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC));
        pvtpoints[i as usize].acceleration = calculate_acceleration(i*(NO_DETAILED_POINTS_PER_SEC/NO_ROUGH_POINTS_PER_SEC));
    }

    for i in 0..NO_TOTAL_POINTS+1 {
        allpoints[i as usize].tick = convert_toint32(i);
        allpoints[i as usize].position = calculate_position(i);
        allpoints[i as usize].velocity = calculate_velocity(i);
        allpoints[i as usize].acceleration = calculate_acceleration(i);
    }
    
    writeln!(file, "tick,position,velocity,acceleration,dpos,dvel,dacc").expect("write failed");

    for i in 0..NO_TOTAL_POINTS+1 {
        if i % (NO_DETAILED_POINTS_PER_SEC / NO_ROUGH_POINTS_PER_SEC) != 0 {
        writeln!(
            file,
            "{},{},{},{}",
            allpoints[i as usize].tick,
            allpoints[i as usize].position,
            allpoints[i as usize].velocity,
            allpoints[i as usize].acceleration
        )
        .expect("write points failed");
        } else {
            writeln!(
                file,
                "{},{},{},{},{},{},{}",
                allpoints[i as usize].tick,
                allpoints[i as usize].position,
                allpoints[i as usize].velocity,
                allpoints[i as usize].acceleration,
                pvtpoints[convert_tousize(i / (NO_DETAILED_POINTS_PER_SEC / NO_ROUGH_POINTS_PER_SEC))].position,
                pvtpoints[convert_tousize(i /(NO_DETAILED_POINTS_PER_SEC / NO_ROUGH_POINTS_PER_SEC))].velocity,
                pvtpoints[convert_tousize(i / (NO_DETAILED_POINTS_PER_SEC / NO_ROUGH_POINTS_PER_SEC))].acceleration,
            )
         .expect("write points failed");
        }
    }
}
