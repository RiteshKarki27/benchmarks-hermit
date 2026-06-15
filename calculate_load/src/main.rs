// use alloc::rc::Rc;
use nix::unistd::{sysconf, SysconfVar};
// use procfs::process::Process;
use procfs::{CurrentSI, KernelStats};
use std::{collections::HashMap, sync::atomic::{AtomicU64, Ordering}, thread, time::{Duration, Instant}};

struct TaskId(i32);
const EXP_1: f64 = 0.920044415;
const EXP_5: f64 = 0.983471454;
const EXP_15: f64 = 0.994459811;

// Variables for hermit kernel. To be tested later 
// enum TaskStatus {
//     Invalid,
//     Ready,
//     Running,
//     Finished,
//     Idle,
// }
//
// struct Priority(u8);
// type CoreId = u32;
//
// struct Task {
//     id: TaskId, 
//     status: TaskStatus,
//     prio: Priority,
//     core_id: CoreId,
//     // stacks: TaskStacks,
// }
#[derive(Default, Clone)]
struct LoadAvg {
    avg_1: f64,
    avg_5: f64, 
    avg_15: f64,
}

impl LoadAvg {
    fn new() -> Self {
        Self { avg_1: 0.0, avg_5: 0.0, avg_15: 0.0}
    }

    fn update_values(&mut self, active_tasks: f64) {
        self.avg_1 = EXP_1 * self.avg_1 + (1.0 - EXP_1) * active_tasks;
        self.avg_5 = EXP_5 * self.avg_5 + (1.0 - EXP_5) * active_tasks;
        self.avg_15 = EXP_15 * self.avg_15 + (1.0 - EXP_15) * active_tasks;
    }
}

static count: AtomicU64 = AtomicU64::new(0);

fn increment() {
    count.fetch_add(1, Ordering::Relaxed);
}

fn get_count() -> u64 {
    count.load(Ordering::Relaxed)
}

fn returnNoCores() -> i64 {
    sysconf(SysconfVar::_NPROCESSORS_ONLN).unwrap().unwrap()
}

fn listTasksWithCpu() -> procfs::ProcResult<()> {
    for proc in procfs::process::all_processes()? {
        let stat = proc.unwrap().stat()?;
        if stat.state == 'R' || stat.state == 'D'{
            println!("PID {} CPU {} CMD {}", stat.pid, stat.processor.unwrap_or(-1), stat.comm);
            increment();
        }
    }
    Ok(())
}

fn active_tasks_per_cpu() -> procfs::ProcResult<HashMap<i32,u64>> {
    let mut counts_per_cpu: HashMap<i32, u64> = HashMap::new();

    for proc in procfs::process::all_processes()? {
        let tasks = proc.unwrap().tasks()?;
        for task in tasks {
            let Ok(task) = task else {continue};
            let task_stat = task.stat()?;
            if matches!(task_stat.state, 'R' | 'D') {
                if let Some(cpu) = task_stat.processor {
                    *counts_per_cpu.entry(cpu).or_insert(0) += 1;
                }
            }

        }
    }
    Ok(counts_per_cpu)
}

fn main() {
    // println!("Hello, world!");

    let mut load_avg = LoadAvg::new();

    while true {
        // thread::sleep(Duration::from_secs(5));
        let no_cores = sysconf(SysconfVar::_NPROCESSORS_ONLN).unwrap().unwrap();
        println!("No of cores {}", returnNoCores());

        // let result = match listTasksWithCpu() {
        //     Ok(()) => println!("Done."), 
        //     Err(e) => eprintln!("Error with {e}"),
        // };
        // println!("Count: {}", get_count());
        let kstats  = procfs::KernelStats::current().unwrap();
        let active_procs = (kstats.procs_running.unwrap_or(0) + kstats.procs_blocked.unwrap_or(0)) as f64;

        let mut per_cpu_load: Vec<LoadAvg> = vec![LoadAvg::default(); returnNoCores() as usize];
        let mut next_tick = Instant::now();

        load_avg.update_values(active_procs);
        println!("load avg:  {:.2} {:.2} {:.2}", load_avg.avg_1, load_avg.avg_5, load_avg.avg_15);

        let count_tasks_per_cpu = active_tasks_per_cpu().unwrap();
        
        for cpu in 0..returnNoCores() {
            let active_tasks = count_tasks_per_cpu.get(&(cpu as i32)).copied().unwrap_or(0) as f64;
            per_cpu_load[cpu as usize].update_values(active_tasks);
        }

        for (cpu, load) in per_cpu_load.iter().enumerate() {
            println!("load avg:  {:.2} {:.2} {:.2}", load.avg_1, load.avg_5, load.avg_15);
        }

        next_tick += std::time::Duration::from_secs(5);
        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        }
    }

}
