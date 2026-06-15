use std::time::Duration;
use std::thread::sleep;
use fork::{fork, Fork};
use crate::utils::{task_work};
mod utils;

const LENGHT: i64 = 256;
const BLOCKING_T: i32 = 2; // s ?
const CORE: usize = 8;


fn main() {
    let nb_tasks = CORE * 2; // or more ?
    let (total_time,blocked_time) = scenario_short_task(nb_tasks);
    let pourc_waiting = ((blocked_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0) as u32;

    println!("-- Running scheduler benchmark --");
    println!("LENGHT: {}    BLOCKING_T: {}      CORE: {}", LENGHT, BLOCKING_T,CORE);
    println!("NB of tasks: {}",nb_tasks);
    println!("Total time : {:?} s", total_time);
    println!("Blocking Time due to scheduling : {:?}",blocked_time);
    println!("Porcentage of blocking time: {:2}%",pourc_waiting);

}


fn scenario_short_task(nb_task: usize) -> (Duration,Duration) {

    //we are going to mesure first how long a short task takes without any contention:
    let start_init = std::time::Instant::now();
    match fork() {
            Ok(Fork::Parent(child)) => {
                let _ = fork::waitpid(child);
            }
            Ok(Fork::Child) => {
                
                task_work(LENGHT, Duration::from_millis(0));
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Fork échoué: {}", e);
            }
    }
    // when the child has finished
    let task_alone_t = start_init.elapsed();

    // now we do the real test
    let mut pids = Vec::new();
    let start = std::time::Instant::now();
    
    for _i in 0..nb_task {
        match fork() {
            Ok(Fork::Parent(child)) => {
                pids.push(child);
                
            }
            Ok(Fork::Child) => {
                
                task_work(LENGHT, Duration::from_millis(0));
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Fork échoué: {}", e);
            }
        }
    }
    
    
    for pid in pids {
        let _ = fork::waitpid(pid);
    }

    // analysing part
    let total_time = start.elapsed();
    let ideal_time = task_alone_t;
    let blocked_time = match total_time.checked_sub(ideal_time) { 
        Some(diff) => diff,
        None => {
            eprintln!("ERR : total_time < ideal_time");
            eprintln!("   total_time: {:.9} s", total_time.as_secs_f64());
            eprintln!("   ideal_time: {:.9} s", ideal_time.as_secs_f64());
            eprintln!("   Différence: {:.9} s", 
                      (ideal_time.as_secs_f64() - total_time.as_secs_f64()));
            Duration::ZERO
        }
    };
    

    (total_time,blocked_time)
}

