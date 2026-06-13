use std::time::Duration;
use std::thread::sleep;


pub fn task_work(n: i64, blocking_time: Duration) {
    // Simulation de travail
    sleep(blocking_time);
    matrix_work(n);
}

fn matrix_work(n: i64) {
    let size = n as usize;
    let mut a = vec![vec![0_i64; size]; size];
    let mut b = vec![vec![0_i64; size]; size];
    let mut c = vec![vec![0_i64; size]; size];
    
    // Init
    for i in 0..size {
        for j in 0..size {
            let idx = (i * size + j) as i64;
            a[i][j] = idx;
            b[i][j] = idx * 2;
        }
    }
    
    // Multiplication
    for i in 0..size {
        for j in 0..size {
            for k in 0..size {
                c[i][j]+= a[i][k] * b[k][j];
            }
        }
    }
}