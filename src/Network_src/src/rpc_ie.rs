use crate::rpc::client::{ApplyFor, Request};
use crate::rpc::service::RPCExecutor;
use crate::rpc::{Database, Event};
use crate::rpc_ie::execute::Execution;
use crate::rpc_ie::parameter::{Args, FutureEx, Instruction};
use crate::OmegaIterator;
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use std::mem;
use std::str::from_utf8;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, RwLock};
use Install_src::element::Node;

pub trait RPC<'life> {
    ///数据库
    fn database() -> Database<'life>;
    //自定义服务
    fn server() -> RPCExecutor<'life, Args<'life>, Args<'life>> {
        RPCExecutor::from(<Self as RPC>::database())
    }
    ///多线程控制
    fn communicate_rwlock(e: usize) -> (Arc<RwLock<Sender<bool>>>, Arc<RwLock<Receiver<bool>>>) {
        let (at, bt) = mpsc::channel::<bool>(e);
        (Arc::new(RwLock::new(at)), Arc::new(RwLock::new(bt)))
    }
    ///自定义链接
    fn client() -> Request<'life> {
        Request::from(<Self as RPC>::database())
    }
}
#[async_trait]
pub trait RxServer: RPC<'static> {
    ///构建
    fn verify<'life>(
        e: (ReadHalf<'life>, WriteHalf<'life>),
    ) -> (BufReader<ReadHalf<'life>>, BufWriter<WriteHalf<'life>>) {
        (BufReader::new(e.0), BufWriter::new(e.1))
    }
    ///设置数据流
    fn set_bitcoin(mut e: String) -> BytesMut {
        let mut x = BytesMut::with_capacity(1024);
        x.put(e.as_bytes());
        x
    }
    ///放入数据流
    async fn get_bitcoin(e: &mut BufReader<ReadHalf>) -> anyhow::Result<BytesMut> {
        let mut x = BytesMut::with_capacity(1024);
        e.read_buf(&mut x).await?;
        Ok(x)
    }
    //链接
    async fn connect_nc(
        future_name: &'static str,
        host: &'static str,
        mut etc: FutureEx<'static, Args<'static>, anyhow::Result<()>>,
    ) -> Execution<'static> {
        Execution::from(vec![spawn(async move {
            let mut mc = TcpStream::connect(host).await?;
            let et = <Self as RPC>::client();
            if let Some(u) = et.0.get(&ApplyFor {
                subscription: future_name,
                function: Default::default(),
            }) {
                let (mut ac, mut ab) = <Self as RxServer>::verify(mc.split());
                let mut er = <Self as RxServer>::set_bitcoin(serde_json::to_string(u)?);
                ab.write_all_buf(&mut er).await?;
                ab.flush().await?;
                let mut er = <Self as RxServer>::get_bitcoin(&mut ac).await?;
                etc.run_sync(serde_json::from_str(from_utf8(unsafe {
                    mem::transmute::<&'_ [u8], &'static [u8]>(&er[..])
                })?)?)
                .await?;
            }
            Ok(())
        })])
    }
    ///#rpc 服务
    async fn server_nc(host: &'static str) -> Execution<'static> {
        Execution::from(vec![spawn(async move {
            let mc = TcpListener::bind(host).await?;
            let et = Arc::new(RwLock::new(<Self as RPC>::server()));
            let (at, bt) = <Self as RPC>::communicate_rwlock(1024);
            loop {
                let dv = et.clone();
                let at = at.clone();
                let bt = bt.clone();
                let (mut ad, _) = mc.accept().await?;
                spawn(async move {
                    let (mut ac, mut ab) = <Self as RxServer>::verify(ad.split());
                    let mut vb = <Self as RxServer>::get_bitcoin(&mut ac).await?;
                    let ApplyFor {
                        subscription,
                        function,
                    } = serde_json::from_str::<ApplyFor>(from_utf8(unsafe {
                        mem::transmute::<&'_ [u8], &'static [u8]>(&vb[..])
                    })?)?;
                    let mut er =
                        if let Some(sv) = dv.read().await.run_nc(function, subscription).await? {
                            if let Ok(e) = sv.parse() {
                                if let Instruction::Backs = e {
                                    //停止
                                    at.read().await.send(true).await?;
                                }
                            }
                            //发送结果
                            <Self as RxServer>::set_bitcoin(serde_json::to_string(&sv)?)
                        } else {
                            <Self as RxServer>::set_bitcoin(serde_json::to_string(&&Args::None)?)
                        };
                    ab.write_all_buf(&mut er).await?;
                    ab.flush().await?;
                    Ok(()) as anyhow::Result<()>
                });
                if let Some(sv) = bt.write().await.recv().await {
                    if sv {
                        break;
                    }
                };
            }
            Ok(())
        })])
    }
}
impl RPC<'static> for Node {
    fn database() -> Database<'static> {
        Database::from(OmegaIterator([Event {
            name: "test",
            args: Default::default(),
            function: FutureEx::sync(async {
                println!("testing");
                Ok(Args::None)
            }),
        }]))
    }
}
impl RxServer for Node {}
///#实现
pub mod comply {
    use crate::rpc::service::{Function, RPCExecutor};
    use crate::rpc::Event;
    use hashbrown::HashMap;
    use serde::{Deserialize, Serialize};
    use std::cmp::Ordering;
    use std::collections::BTreeMap;
    use std::hash::{Hash, Hasher};
    use std::ops::{Deref, DerefMut};

    impl<'life, RE, GX> PartialOrd<Self> for Event<'life, RE, GX>
    where
        GX: Send + Sized + Sync,
        RE: Deserialize<'life> + Serialize,
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.name.partial_cmp(other.name)
        }
    }
    impl<'life, RE, GX> Ord for Event<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            self.name.cmp(other.name)
        }
    }
    impl<'life, RE, GX> Eq for Event<'life, RE, GX>
    where
        GX: Send + Sized + Sync,
        RE: Deserialize<'life> + Serialize,
    {
    }
    impl<'life, RE, GX> PartialEq for Event<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn eq(&self, other: &Self) -> bool {
            self.name.eq(other.name)
        }
    }
    impl<'life, RE, GX> Hash for Event<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.name.hash(state);
        }
    }
    impl<'life, RE, GX> Into<HashMap<&'life str, Function<'life, RE, anyhow::Result<GX>>>>
        for RPCExecutor<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn into(self) -> HashMap<&'life str, Function<'life, RE, anyhow::Result<GX>>> {
            let mut et = HashMap::new();
            self.0.into_iter().for_each(|(a, b)| {
                et.insert(a, b);
            });
            et
        }
    }
    impl<'life, RE, GX> Hash for RPCExecutor<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state)
        }
    }
    impl<'life, RE, GX> Default for RPCExecutor<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn default() -> Self {
            RPCExecutor(BTreeMap::default())
        }
    }

    impl<'life, RE, GX> DerefMut for RPCExecutor<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<'life, RE, GX> Deref for RPCExecutor<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        type Target = BTreeMap<&'life str, Function<'life, RE, anyhow::Result<GX>>>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
///参数
pub mod parameter {
    use crate::Async;
    use bytes::BytesMut;
    use serde::{Deserialize, Serialize};
    use std::future::Future;

    //执行闭包
    pub enum FutureEx<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        ///异步函数
        AsyncTrait(Async<'life, GX>),
        ///异步闭包函数
        AsyncFnTrait(Box<dyn FnMut(RE) -> Async<'life, GX> + Send + Sync + 'life>),
        //普通函数
        FnTrait(Box<dyn FnMut(RE) -> GX + Send + Sync + 'life>),
    }
    impl<'life, RE, GX> FutureEx<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life> + 'life,
        GX: Sized + Send + 'life,
    {
        //FutureEx::AsyncTrait(Box::pin(e))
        pub fn sync(e: impl Future<Output = GX> + Send + Sync + 'life) -> Self {
            FutureEx::AsyncTrait(Box::pin(e))
        }
        //FutureEx::FnTrait(Box::new(e))
        pub fn def(e: fn(RE) -> GX) -> Self {
            FutureEx::FnTrait(Box::new(e))
        }
        //FutureEx::AsyncFnTrait(Box::new(e))
        pub fn def_sync(e: fn(RE) -> Async<'life, GX>) -> Self {
            FutureEx::AsyncFnTrait(Box::new(e))
        }
        pub async fn run_sync(&mut self, arg: RE) -> GX {
            match self {
                FutureEx::AsyncTrait(e) => e.await,
                FutureEx::AsyncFnTrait(e) => e(arg).await,
                FutureEx::FnTrait(e) => e(arg),
            }
        }
    }

    impl<'life, RE, GX> From<Box<dyn FnMut(RE) -> GX + 'life + Send + Sync>> for FutureEx<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn from(value: Box<dyn FnMut(RE) -> GX + Send + Sync + 'life>) -> Self {
            FutureEx::FnTrait(value)
        }
    }
    impl<'life, RE, GX> From<Box<dyn FnMut(RE) -> Async<'life, GX> + Send + Sync + 'life>>
        for FutureEx<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn from(value: Box<dyn FnMut(RE) -> Async<'life, GX> + Send + Sync + 'life>) -> Self {
            FutureEx::AsyncFnTrait(value)
        }
    }
    impl<'life, RE, GX> From<Async<'life, GX>> for FutureEx<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn from(value: Async<'life, GX>) -> Self {
            FutureEx::AsyncTrait(value)
        }
    }

    ///#通用参数
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub enum Args<'life> {
        Byte(Vec<u8>),
        Bit(&'life [u8]),
        String(String),
        Str(&'life str),
        None,
    }
    impl<'life> Into<BytesMut> for Args<'life> {
        fn into(self) -> BytesMut {
            match self {
                Args::Byte(e) => BytesMut::from(e.as_slice()),
                Args::Bit(e) => BytesMut::from(e),
                Args::String(e) => BytesMut::from(e.as_bytes()),
                Args::Str(e) => BytesMut::from(e),
                Args::None => BytesMut::new(),
            }
        }
    }

    impl<'life> Args<'life> {
        ///解析
        pub fn parse(&self) -> anyhow::Result<Instruction> {
            Ok(serde_json::from_str(self.to_string().as_str())?)
        }
    }
    impl<'life> ToString for Args<'life> {
        fn to_string(&self) -> String {
            match self {
                Args::Byte(e) => String::from_utf8(e.to_vec()).unwrap(),
                Args::Bit(e) => std::str::from_utf8(e).unwrap().to_string(),
                Args::String(e) => e.to_string(),
                Args::Str(e) => e.to_string(),
                _ => String::default(),
            }
        }
    }
    ///指令集
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Instruction {
        ///停止
        Backs,
        ///开始
        Start,
        ///暂停
        Pause,
    }
    impl<'life> Default for Args<'life> {
        fn default() -> Self {
            Args::None
        }
    }
}
///#执行异步池
pub mod execute {
    use core::iter::IntoIterator;
    use std::cell::UnsafeCell;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::vec::IntoIter;
    use tokio::task::JoinHandle;
    use Gui_src::{Colour, Information, View};
    ///显示
    pub static THREAD_DISPLAY: AtomicBool = AtomicBool::new(false);
    ///#执行
    pub struct Execution<'life> {
        pub service: Vec<JoinHandle<anyhow::Result<()>>>,
        pub name: UnsafeCell<&'life str>,
    }

    impl<'life> Execution<'life> {
        ///执行
        pub async fn execute(self) -> anyhow::Result<()> {
            if THREAD_DISPLAY.load(Ordering::SeqCst) {
                for a in self.service {
                    unsafe {
                        println!(
                            "{}",
                            Colour::Output.table(Information {
                                list: ["execution"],
                                data: [[*self.name.get()]],
                            })
                        )
                    };
                    a.await??;
                }
            } else {
                for i in self.into_iter() {
                    i.await??;
                }
            }
            Ok(())
        }
    }

    impl<'life> From<Vec<JoinHandle<anyhow::Result<()>>>> for Execution<'life> {
        fn from(value: Vec<JoinHandle<anyhow::Result<()>>>) -> Self {
            Execution {
                service: value,
                name: UnsafeCell::new("default"),
            }
        }
    }

    impl<'life> From<(Vec<JoinHandle<anyhow::Result<()>>>, &'life str)> for Execution<'life> {
        fn from(value: (Vec<JoinHandle<anyhow::Result<()>>>, &'life str)) -> Self {
            Execution {
                service: value.0,
                name: UnsafeCell::new(value.1),
            }
        }
    }

    impl<'life> IntoIterator for Execution<'life> {
        type Item = JoinHandle<anyhow::Result<()>>;
        type IntoIter = IntoIter<JoinHandle<Result<(), anyhow::Error>>>;
        fn into_iter(self) -> Self::IntoIter {
            self.service.into_iter()
        }
    }
}
