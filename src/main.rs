#![warn(non_snake_case)]

mod achieve;
mod beginning;

use crate::achieve::achieve;
use crate::beginning::initial_state::initial;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use tokio::{main, spawn};
use Gui_src::{Colour, Information, View};
use Install_src::Install;
use LinuxService::{Execution as Exec, ExecutionEX};
#[main]
pub async fn main() -> anyhow::Result<()> {
    let start = Arc::new((Mutex::new(false), Condvar::new()));
    let start_node = start.clone();
    ExecutionEX {
        name: Colour::Output.table(Information {
            list: ["ExecutionType"],
            data: [["main"]],
        }),
        executor: [
            (
                spawn(async move {
                    initial(start).await?;
                    achieve().await?;
                    Ok(())
                }),
                "Construct",
            ),
            (
                spawn(async move {
                    let (x, y) = &*start_node;
                    if *y.wait(x.lock().unwrap()).unwrap() {
                        CHILD_THREAD.store(true, Ordering::Release);
                    }
                    Ok(())
                }),
                "NodeCommunication",
            ),
        ],
    }
    .async_run(THREAD_DISPLAY.load(Ordering::SeqCst))
    .await?;
    Ok(())
}
///#设置
pub static PUT_UP: OnceCell<Install> = OnceCell::new();
///线程名称显示
pub static THREAD_DISPLAY: AtomicBool = AtomicBool::new(false);
///子线程执行
pub static CHILD_THREAD: AtomicBool = AtomicBool::new(false);
///线程计数
pub static MASTER_ID: AtomicI64 = AtomicI64::new(0);
///节点安装包
pub const SLAVE_PKG: &str = "./TitansService";
