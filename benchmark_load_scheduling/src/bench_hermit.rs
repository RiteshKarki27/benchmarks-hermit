use crate::utils::{ task_work, analyze_metrics };
mod utils;
use std::env;

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use std::time::{Duration,SystemTime, UNIX_EPOCH, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[cfg(target_os = "hermit")]
use hermit as _;


const CORE: usize = 6;

#[derive(Debug, Serialize)]
struct BenchmarkOutput {
    scenario: String,
    nb_tasks: usize,
    length: i64,
    length_short: Option<i64>,
    length_long: Option<i64>,
    io_time_ms: u64,
    io_time_short_ms: u64,
    cores: usize,
    total_time_ms: u128,
    waiting_time: Stats,
    execution_time: Stats,
    response_time: Stats,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Stats {
    min: u64,
    max: u64,
    mean: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessMetrics {
    tag: u8,
    index: usize,
    creation_time: u64,
    start_work_time: u64,
    end_work_time: u64,
    waiting_time: u64,
    execution_time: u64,
    response_time: u64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let nb_tasks: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(24);
    let length: i64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(512);
    let io_time: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(600);
    let mode: &str = args.get(4).map(|s| s.as_str()).unwrap_or("mix");

    match mode {
        "fix" => {
            let (total_time, waiting, execution, response) = 
                scenario_task(nb_tasks, length, Duration::from_millis(io_time));
            
            let output = BenchmarkOutput {
                scenario: "fix".to_string(),
                nb_tasks,
                length,
                length_short: None,
                length_long: None,
                io_time_ms: io_time,
                io_time_short_ms: 0,
                cores: CORE,
                total_time_ms: total_time.as_micros(),
                waiting_time: waiting,
                execution_time: execution,
                response_time: response,
            };
            
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        "mix" => {
            let (total_time, waiting, execution, response) = scenario_mix(
                nb_tasks,
                length / 2,
                length,
                Duration::from_millis(io_time),
                Duration::from_millis(0)
            );
            
            let output = BenchmarkOutput {
                scenario: "mix".to_string(),
                nb_tasks,
                length: 0,
                length_short: Some(length / 2),
                length_long: Some(length),
                io_time_ms: io_time,
                io_time_short_ms: 0,
                cores: CORE,
                total_time_ms: total_time.as_micros(),
                waiting_time: waiting,
                execution_time: execution,
                response_time: response,
            };
            
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            eprintln!("Unknown mode: {}. Use 'fix' or 'mix'", mode);
            std::process::exit(1);
        }
    }
}


fn scenario_mix(
    nb_task: usize,
    length_short: i64,
    length_long: i64,
    blocking_time_l: Duration,
    blocking_time_s: Duration
) -> (Duration, Stats, Stats, Stats) {
    let mut tids = Vec::new();
    let metrics_storage = Arc::new(Mutex::new(Vec::<ProcessMetrics>::new()));
    let start = Instant::now();

    for i in 0..nb_task {
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        let storage_clone = Arc::clone(&metrics_storage);
            let tid = thread::spawn( move || { // give ownership to the thread or not compile...
            // each thread as a copy of i 
            // This is what is executed in the child                
                let start_work_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64;

                if i % 2 == 0 {
                    task_work(length_short, blocking_time_s);
                } else {
                    task_work(length_long, blocking_time_l);
                }

                let end_work_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64;

                let waiting_time = start_work_time - creation_time;
                let execution_time = end_work_time - start_work_time;
                let response_time = end_work_time - creation_time;

                let metrics = ProcessMetrics {
                    tag: if i % 2 == 0 { 0 } else { 1 },
                    index: i,
                    creation_time,
                    start_work_time,
                    end_work_time,
                    waiting_time,
                    execution_time,
                    response_time,
                };
                storage_clone.lock().unwrap().push(metrics);
                
            });
        tids.push(tid);
    }
    
    for tid in tids {
        tid.join().unwrap();
    }

    let total_time = start.elapsed();

    let all_metrics = metrics_storage.lock().unwrap();
    let mut timing_metric = HashMap::new();
    for metrics in all_metrics.iter() {
    timing_metric.insert(metrics.index, metrics.clone());
    }
   
    let (waiting, execution, response) = analyze_metrics(timing_metric);

    (total_time, waiting, execution, response)
}


fn scenario_task(
    nb_task: usize,
    length: i64,
    blocking_time: Duration
) -> (Duration, Stats, Stats, Stats) {
    let mut tids = Vec::new();
    let metrics_storage = Arc::new(Mutex::new(Vec::<ProcessMetrics>::new()));
    let start = Instant::now();

    for i in 0..nb_task {
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        let storage_clone = Arc::clone(&metrics_storage);
        let tid = thread::spawn(move || {
            let start_work_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;

            task_work(length, blocking_time);

            let end_work_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;

            let waiting_time = start_work_time - creation_time;
            let execution_time = end_work_time - start_work_time;
            let response_time = end_work_time - creation_time;

            let metrics = ProcessMetrics {
                tag: 2,
                index: i,
                creation_time,
                start_work_time,
                end_work_time,
                waiting_time,
                execution_time,
                response_time,
            };
            storage_clone.lock().unwrap().push(metrics);
        });
        tids.push(tid);
    }

    for tid in tids {
        tid.join().unwrap();
    }

    let total_time = start.elapsed();

    let all_metrics = metrics_storage.lock().unwrap();
    let mut timing_metric = HashMap::new();
    for metrics in all_metrics.iter() {
        timing_metric.insert(metrics.index, metrics.clone());
    }

    let (waiting, execution, response) = analyze_metrics(timing_metric);

    (total_time, waiting, execution, response)
}