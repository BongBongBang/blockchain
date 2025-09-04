#![allow(dead_code)]
mod block;
mod blockchain;
mod cli;
mod proof_of_work;
mod transaction;
mod wallet;
mod wallets;
mod tx;
mod utxo;

use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::cli::CommandLine;

/*
全局程序结束回调函数变量
 */
static EXIT_CALLBACKS: Lazy<Mutex<Vec<Box<dyn FnOnce() + Send>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

static _AT_EXIT_MONITOR: Lazy<AtExitMonitor> = Lazy::new(|| AtExitMonitor);

/*
注册全局结束回调函数
 */
pub fn register_exit_callback<F>(cb: F)
where
    F: FnOnce() + Send + 'static,
{
    let mut callbacks = EXIT_CALLBACKS.lock().unwrap();
    callbacks.push(Box::new(cb));
}

/*
执行回调函数
*/
fn run_exit_callbacks() {
    let mut cds_guard = EXIT_CALLBACKS.lock().unwrap();
    while let Some(cb) = cds_guard.pop() {
        cb();
    }
}

/*
全局程序结束监控器
 */
struct AtExitMonitor;
impl Drop for AtExitMonitor {
    fn drop(&mut self) {
        run_exit_callbacks();
    }
}

fn main() {
    let _exit_hook = AtExitMonitor;
    let mut command_line = CommandLine::new();
    command_line.run();
}
