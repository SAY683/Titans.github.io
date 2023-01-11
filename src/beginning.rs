///#执行阶段
pub mod beginning_of {}

///#初始阶段
pub mod initial_state {
    use crate::{PUT_UP, SLAVE_PKG, THREAD_DISPLAY};
    use ftlog::appender::{Duration, FileAppender, Period};
    use ftlog::{info, FtLogFormatter, LevelFilter};
    use hashbrown::HashMap;
    use std::path::Path;
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Condvar, Mutex};
    use tokio::spawn;
    use tokio::task::JoinHandle;
    use Gui_src::{Colour, Information, View};
    use Install_src::{Install, InstallSrc};
    use LinuxService::{Execution, ExecutionEX};
    use Network_src::ssh_sc::SshSc;

    pub async fn initial(start_main: Arc<(Mutex<bool>, Condvar)>) -> anyhow::Result<()> {
        let start = start_main.clone();
        ExecutionEX {
            name: Colour::Output.table(Information {
                list: ["ExecutionType"],
                data: [["initial"]],
            }),
            executor: [
                (
                    spawn(async move {
                        let (mutex, condvar) = &*start;
                        construct()?;
                        //logs_to()?;
                        *mutex.lock().unwrap() = true;
                        condvar.notify_all();
                        Ok(())
                    }),
                    "",
                ),
                (
                    spawn(async move {
                        let (x, y) = &*start_main;
                        if *y.wait(x.lock().unwrap()).unwrap() {
                            if PUT_UP.get().unwrap().drive.master.cluster_enabled {
                                node_build().await?;
                            } else {
                            }
                        }
                        Ok(())
                    }),
                    "",
                ),
            ],
        }
        .async_run(THREAD_DISPLAY.load(Ordering::SeqCst))
        .await?;
        Ok(())
    }

    ///#构建
    fn construct() -> anyhow::Result<()> {
        PUT_UP.get_or_init(|| Colour::error_display("PUT_UP", Install::build_settings()));
        Ok(())
    }
    ///#节点构建
    async fn node_build() -> anyhow::Result<()> {
        let mut x: Vec<JoinHandle<anyhow::Result<()>>> = vec![];
        for i in PUT_UP.get().unwrap().drive.slave.iter() {
            x.push(spawn(async move {
                let mut t = String::new();
                if PUT_UP.get().unwrap().drive.master.scp_build {
                    t = format!(
                        "ls -la {}",
                        PUT_UP
                            .get()
                            .unwrap()
                            .data
                            .storage
                            .node_data
                            .to_str()
                            .unwrap()
                    )
                }
                for (x, (_a, b)) in i
                    .shell(vec![
                        format!(
                            "chmod -R 777 {}",
                            PUT_UP
                                .get()
                                .unwrap()
                                .data
                                .storage
                                .node_data
                                .to_str()
                                .unwrap()
                        )
                        .as_str(),
                        //驱动执行
                        format!(
                            "cd {} && {}",
                            PUT_UP
                                .get()
                                .unwrap()
                                .data
                                .storage
                                .node_data
                                .to_str()
                                .unwrap(),
                            PUT_UP
                                .get()
                                .unwrap()
                                .data
                                .storage
                                .node_service_path
                                .to_str()
                                .unwrap()
                        )
                        .as_str(),
                        &t,
                    ])
                    .unwrap_or_else(|_| {
                        println!(
                            "{}",
                            Colour::Debug.table(Information {
                                list: ["Node", "PossibleError"],
                                data: [
                                    [&i.node_name, "TimeOut"],
                                    [&i.node_name, "FileConfigurationError"],
                                    [&i.node_name, "SessionMisconfiguration"]
                                ],
                            })
                        );
                        HashMap::default()
                    })
                    .iter()
                    .enumerate()
                {
                    if x != 0 {
                        println!("{b}");
                    } else if b.is_empty() {
                        let mut x = PUT_UP.get().unwrap().data.storage.node_data.to_path_buf();
                        x.pop();
                        i.scp(Path::new(SLAVE_PKG), Path::new(&x), true)?;
                        println!(
                            "{}",
                            Colour::Output.table(Information {
                                list: ["Node", "Local_Path", "Remote_Path"],
                                data: [[&i.node_name, SLAVE_PKG, x.to_str().unwrap()]]
                            })
                        );
                    }
                }
                info!("{}-->{}Node_Build<+OK+>", i.node_name, i.host);
                Ok(())
            }));
        }
        for i in x {
            i.await??;
        }
        Ok(())
    }
    ///#日志
    #[warn(dead_code)]
    fn logs_to() -> anyhow::Result<()> {
        //错误
        #[warn(unreachable_code)]
        let x = &Colour::operation("PUT_UP", PUT_UP.get()).data.logs;
        if x.enabled {
            core::unreachable!();
            ftlog::builder()
                .format(FtLogFormatter)
                .max_log_level(LevelFilter::Info)
                .root(FileAppender::rotate_with_expire(
                    x.path.as_os_str(),
                    Period::Day,
                    Duration::days(x.expiration_day),
                ))
                .build()?
                .init()?;
        }
        Ok(())
    }
}
