mod construct;
mod examination;

use crate::construct::{build, start_link};
use arc_swap::ArcSwap;
use async_trait::async_trait;
use comfy_table::Table;
use lazy_static::lazy_static;
use once_cell::sync::{Lazy, OnceCell};
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use tokio::task::JoinHandle;
use tokio::{main, spawn};
use Gui_src::{Colour, Information, View};
use Install_src::element::Node;
use Install_src::{Install, InstallSrc};

#[main]
//#[cfg(target_os = "TitansService")]
async fn main() -> anyhow::Result<()> {
    let start = Arc::new((Mutex::new(false), Condvar::new()));
    let start_node = start.clone();
    ExecutionEX {
        name: Colour::Output.table(Information {
            list: ["ExecutiveService"],
            data: [["main"]],
        }),
        executor: [
            (
                spawn(async move {
                    build(start).await?;
                    start_link().await?;
                    Ok(())
                }),
                "build",
            ),
            (
                spawn(async move {
                    let (x, y) = &*start_node;
                    if *y.wait(x.lock().unwrap()).unwrap() {}
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

///线程名称显示
pub static THREAD_DISPLAY: AtomicBool = AtomicBool::new(false);
///程序节点名称
pub static NODE_NAME: OnceCell<String> = OnceCell::new();

///#线程池
pub struct ExecutionEX<'life, const RE: usize, GX: Sized> {
    pub name: Table,
    pub executor: [(JoinHandle<anyhow::Result<GX>>, &'life str); RE],
}

#[async_trait]
pub trait Execution<GX: Sized> {
    type View;
    async fn async_run(self, view: Self::View) -> anyhow::Result<Vec<GX>>;
    fn run(self, view: Self::View);
}

#[async_trait]
impl<'life, const RE: usize, GX: Sized + Send + Sync> Execution<GX> for ExecutionEX<'life, RE, GX> {
    type View = bool;
    async fn async_run(self, view: Self::View) -> anyhow::Result<Vec<GX>> {
        let mut r = vec![];
        if view {
            println!("{}", self.name);
        };
        for (x, y) in self.executor.into_iter() {
            if view {
                println!("{y}");
                r.push(Colour::error_display(y, x.await?));
            } else {
                match x.await? {
                    Ok(x) => {
                        r.push(x);
                    }
                    Err(r) => {
                        println!(
                            "{}",
                            Colour::Debug.logs_is(y).table(Information {
                                list: ["ExecutionError"],
                                data: [[format!("{r:?}").as_str()]],
                            })
                        );
                    }
                }
            }
        }
        Ok(r)
    }

    fn run(self, view: Self::View) {
        self.executor.map(|(a, b)| {
            if view {
                println!("{b}");
                a
            } else {
                Colour::Output.logs_is(b);
                a
            }
        });
    }
}

///#线程
pub mod exec {
    use super::*;
    use crate::ExecutionEX;

    impl<'life, const RE: usize, GX: Sized> From<[(JoinHandle<anyhow::Result<GX>>, &'life str); RE]>
        for ExecutionEX<'life, RE, GX>
    {
        fn from(value: [(JoinHandle<anyhow::Result<GX>>, &'life str); RE]) -> Self {
            ExecutionEX {
                name: Default::default(),
                executor: value,
            }
        }
    }

    impl<'life, const RE: usize, GX: Sized>
        From<(Table, [(JoinHandle<anyhow::Result<GX>>, &'life str); RE])>
        for ExecutionEX<'life, RE, GX>
    {
        fn from(value: (Table, [(JoinHandle<anyhow::Result<GX>>, &'life str); RE])) -> Self {
            ExecutionEX {
                name: value.0,
                executor: value.1,
            }
        }
    }
}
lazy_static! {
    pub static ref LOCAL_IP: anyhow::Result<String> = {
        let x = UdpSocket::bind("0.0.0.0:0")?;
        x.connect("8.8.8.8:80")?;
        Ok(x.local_addr()?.ip().to_string())
    };
    pub static ref LOCAL_HOST: anyhow::Result<SocketAddr> = {
        let x = UdpSocket::bind("0.0.0.0:0")?;
        x.connect("8.8.8.8:80")?;
        Ok(x.local_addr()?)
    };
    pub static ref LOCAL_PORT: anyhow::Result<String> = {
        let x = UdpSocket::bind("0.0.0.0:0")?;
        x.connect("8.8.8.8:80")?;
        Ok(x.local_addr()?.port().to_string())
    };
    pub static ref NODE_HOST: anyhow::Result<SocketAddr> = {
        Ok(SocketAddr::new(
            NODE.get().unwrap().host.parse::<SocketAddr>()?.ip(),
            INSTALL_SRC
                .load()
                .drive
                .master
                .host
                .parse::<SocketAddr>()?
                .port(),
        ))
    };
    pub static ref MASTER_HOST: anyhow::Result<SocketAddr> =
        Ok(INSTALL_SRC.load().drive.master.host.parse::<SocketAddr>()?);
}
///节点数据
pub static NODE: OnceCell<Node> = OnceCell::new();
///设置数据
pub static INSTALL_SRC: Lazy<ArcSwap<Install>> =
    Lazy::new(|| ArcSwap::from_pointee(Install::build_settings().unwrap()));
