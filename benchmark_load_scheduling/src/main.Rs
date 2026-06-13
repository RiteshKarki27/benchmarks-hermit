use std::time::Duration;
use std::thread::sleep;
use fork::{fork, Fork};
use crate::utils::{task_work};
mod utils;

const LENGHT: i64 = 256;
const BLOCKING_T: i32 = 2;
const CORE: usize = 4;

fn main() {
    let nb_tasks = CORE * 2; // ?
    let duration = scenario_short_task(nb_tasks);
    println!("{}.{}.{}", duration.as_secs(), duration.subsec_millis(), duration.subsec_micros() % 1000);
}


fn scenario_short_task(nb_task: usize) -> Duration {
    let start = std::time::Instant::now();
    let mut pids = Vec::new();
    
    for i in 0..nb_task {
        match fork() {
            Ok(Fork::Parent(child)) => {
                pids.push(child);
                println!("Fork : {}", child);
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
    
    start.elapsed()
}

