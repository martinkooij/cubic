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
    const NO_ROUGH_POINTS_PER_SEC: i32 = 2;
    const MAX_VELOCITY: f64 = 5.0;
    const MAX_ACCELERATION: f64 = 2.0;
    const VELOCITY_UP: f64 = 2.0;
    const ACCELERATION_UP: f64 = MAX_ACCELERATION;
    const VELOCITY_FLAT: f64 = 6.0;
    const ACCELERATION_DOWN: f64 = -MAX_ACCELERATION;
    const VELOCITY_DOWN: f64 = 2.0;
    const TOTAL_TICKS: i32 = 10;
    const NO_MAIN_POINTS: i32 = TOTAL_TICKS * NO_ROUGH_POINTS_PER_SEC;
    const NO_TOTAL_POINTS: i32 = TOTAL_TICKS * NO_DETAILED_POINTS_PER_SEC;
    const START_POSITION: f64 = 0.0;
    const START_VELOCITY: f64 = 0.0;

    #[derive(Debug, Copy, Clone)]
    struct Pvtpoint {
        tick: i32,
        position: f64,
        velocity: f64,
        acceleration: f64,
    }

    let mut _pvtpoints: [Pvtpoint; NO_MAIN_POINTS as usize] = [Pvtpoint {
        tick: 0,
        position: START_POSITION,
        velocity: START_VELOCITY,
        acceleration: ACCELERATION_UP,
    }; NO_MAIN_POINTS as usize];

    let mut allpoints: [Pvtpoint; NO_TOTAL_POINTS as usize] = [Pvtpoint {
        tick: 0,
        position: START_POSITION,
        velocity: START_VELOCITY,
        acceleration: ACCELERATION_UP,
    }; NO_TOTAL_POINTS as usize];

    // set up files
    let file = File::create("allpoints.csv").expect("Unable to create file");
    let mut file = LineWriter::new(file);

    // Accelerate up
    for i in 1..((VELOCITY_UP as i32) * NO_DETAILED_POINTS_PER_SEC) {
        allpoints[i as usize].tick = convert_toint32(i);
        allpoints[i as usize].position = allpoints[convert_tousize(i - 1)].position
            + (allpoints[convert_tousize(i - 1)].velocity * 1.0
                / NO_DETAILED_POINTS_PER_SEC as f64)
            + (0.5
                * allpoints[convert_tousize(i - 1)].acceleration
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64)
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64));
        allpoints[i as usize].velocity = allpoints[convert_tousize(i - 1)].velocity
            + allpoints[convert_tousize(i - 1)].acceleration
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64);
        allpoints[i as usize].acceleration = ACCELERATION_UP;
    }

    // Flat velocity
    for i in (VELOCITY_UP as i32) * NO_DETAILED_POINTS_PER_SEC
        ..((VELOCITY_UP as i32 + VELOCITY_FLAT as i32) * NO_DETAILED_POINTS_PER_SEC)
    {
        allpoints[i as usize].tick = convert_toint32(i);
        allpoints[i as usize].position = allpoints[convert_tousize(i - 1)].position
            + (allpoints[convert_tousize(i - 1)].velocity * 1.0
                / NO_DETAILED_POINTS_PER_SEC as f64)
            + (0.5
                * allpoints[convert_tousize(i - 1)].acceleration
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64)
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64));
        allpoints[i as usize].velocity = allpoints[convert_tousize(i - 1)].velocity
            + allpoints[convert_tousize(i - 1)].acceleration
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64);
        allpoints[i as usize].acceleration = 0.0;
    }

    //Decelerate
    for i in ((VELOCITY_UP as i32 + VELOCITY_FLAT as i32) * NO_DETAILED_POINTS_PER_SEC)
        ..((VELOCITY_UP as i32 + VELOCITY_FLAT as i32 + VELOCITY_DOWN as i32)
            * NO_DETAILED_POINTS_PER_SEC)
    {
        allpoints[i as usize].tick = convert_toint32(i);
        allpoints[i as usize].position = allpoints[convert_tousize(i - 1)].position
            + (allpoints[convert_tousize(i - 1)].velocity * 1.0
                / NO_DETAILED_POINTS_PER_SEC as f64)
            + (0.5
                * allpoints[convert_tousize(i - 1)].acceleration
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64)
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64));
        allpoints[i as usize].velocity = allpoints[convert_tousize(i - 1)].velocity
            + allpoints[convert_tousize(i - 1)].acceleration
                * (1.0 / NO_DETAILED_POINTS_PER_SEC as f64);
        allpoints[i as usize].acceleration = ACCELERATION_DOWN;
    }

    writeln!(file, "tick,position,velocity,acceleration").expect("write failed");

    for i in 0..NO_TOTAL_POINTS {
        writeln!(
            file,
            "{},{},{},{}",
            allpoints[i as usize].tick,
            allpoints[i as usize].position,
            allpoints[i as usize].velocity,
            allpoints[i as usize].acceleration
        )
        .expect("write points failed");
    }
}
