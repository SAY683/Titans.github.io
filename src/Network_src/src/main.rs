#![feature(
    associated_type_defaults,
    generic_const_exprs,
    async_closure,
    inherent_associated_types,
    exclusive_wrapper,
    once_cell
)]
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use std::array;
use std::future::Future;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch, RwLock};

pub mod rpc;
pub mod rpc_ie;
pub mod ssh_sc;

use crate::rpc_ie::execute::Execution;
use crate::rpc_ie::parameter::{Args, FutureEx};
use crate::rpc_ie::RxServer;
use rayon::iter::ParallelIterator;
use tokio::sync::broadcast;
use tokio::{main, spawn};
use Install_src::element::Node;

#[main]
async fn main() -> anyhow::Result<()> {
    let (a, b) = kanal::bounded_async(12);
    let x = Arc::new(RwLock::new(a));
    let y = Arc::new(RwLock::new(b));
    x.read().await.send(true).await?;
    if y.read().await.recv().await? {
        Execution::from(vec![
            spawn(async {
                Node::server_nc("127.0.0.1:8964").await.execute().await?;
                Ok(())
            }),
            spawn(async {
                Node::connect_nc(
                    "test",
                    "127.0.0.1:8964",
                    FutureEx::def(|e| {
                        println!("{e:#?}");
                        Ok(())
                    }),
                )
                .await
                .execute()
                .await?;
                Ok(())
            }),
        ])
        .execute()
        .await?;
    }
    Ok(())
}
///异步
pub type Async<'life, RE> = Pin<Box<dyn Future<Output = RE> + Send + Sync + 'life>>;
///可变缓存流
pub type VariableCacheFlow<GP> = (kanal::Sender<GP>, kanal::Receiver<GP>);
///异步广播
pub type AsyncBroadcast<GP> = (broadcast::Sender<GP>, broadcast::Receiver<GP>);
///异步领导
pub type AsyncLeader<GP> = (mpsc::Sender<GP>, mpsc::Receiver<GP>);
///异步监控
pub type AsyncMonitor<GP> = (watch::Sender<GP>, watch::Receiver<GP>);
///异步通信
pub type AsyncCommunicate<GP> = (oneshot::Sender<GP>, oneshot::Receiver<GP>);
///线程返回
pub type ThreadReturns<'life> = anyhow::Result<Args<'life>>;
///核心迭代器
pub struct OmegaIterator<GP: Sized, const GN: usize>([GP; GN]);

impl<GP: Sized, const GN: usize> OmegaIterator<GP, GN> {
    ///迭代
    pub fn into_init(self, i: fn(GP) -> ())
    where
        GP: Send,
    {
        self.0.into_par_iter().for_each(|e| i(e));
    }
    ///引用迭代
    pub fn iter_init(&self, i: fn(&GP) -> ())
    where
        GP: Send + Sync,
    {
        self.0.par_iter().for_each(|e| i(e))
    }
    ///转换vec
    pub fn to_vec(self) -> Vec<GP>
    where
        GP: Clone,
    {
        self.0.to_vec()
    }
}

impl<GP: Sized, const GN: usize> IntoIterator for OmegaIterator<GP, GN> {
    type Item = GP;
    type IntoIter = array::IntoIter<Self::Item, GN>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<GP: Sized, const GN: usize> From<Vec<GP>> for OmegaIterator<GP, GN> {
    fn from(value: Vec<GP>) -> Self {
        let mut key: [GP; GN] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut y = 0;
        value.into_iter().for_each(|x| {
            key[y] = x;
            y += 1;
        });
        OmegaIterator(key)
    }
}

impl<GP: Sized, const GN: usize> AsRef<[GP]> for OmegaIterator<GP, GN> {
    fn as_ref(&self) -> &[GP] {
        self.0.as_ref()
    }
}
impl<GP: Sized, const GN: usize> AsMut<[GP]> for OmegaIterator<GP, GN> {
    fn as_mut(&mut self) -> &mut [GP] {
        self.0.as_mut()
    }
}
impl<GP: Sized, const GN: usize> From<[GP; GN]> for OmegaIterator<GP, GN> {
    fn from(value: [GP; GN]) -> Self {
        OmegaIterator(value)
    }
}
