use crate::rpc_ie::parameter::{Args, FutureEx};
use crate::{OmegaIterator, ThreadReturns};
use hashbrown::HashSet;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

///数据仓库
pub struct Database<'life>(pub BTreeSet<Event<'life, Args<'life>, ThreadReturns<'life>>>);
impl<'life, const NP: usize>
    From<OmegaIterator<Event<'life, Args<'life>, ThreadReturns<'life>>, NP>> for Database<'life>
{
    fn from(value: OmegaIterator<Event<'life, Args<'life>, ThreadReturns<'life>>, NP>) -> Self {
        let mut et = BTreeSet::new();
        value.into_iter().for_each(|x| {
            et.insert(x);
        });
        Database(et)
    }
}

impl<'life> Into<HashSet<Event<'life, Args<'life>, ThreadReturns<'life>>>> for Database<'life> {
    fn into(self) -> HashSet<Event<'life, Args<'life>, ThreadReturns<'life>>> {
        let mut er = HashSet::new();
        self.0.into_iter().for_each(|x| {
            er.insert(x);
        });
        er
    }
}
///通用
pub struct Event<'life, RE, GX>
where
    RE: Serialize + Deserialize<'life>,
    GX: Sized + Send + Sync,
{
    ///名称
    pub name: &'life str,
    ///参数
    pub args: Args<'life>,
    ///函数
    pub function: FutureEx<'life, RE, GX>,
}
pub mod service {
    use super::*;
    use crate::rpc_ie::parameter::{Args, FutureEx};
    use crate::{Async, OmegaIterator};
    use std::cmp::Ordering;
    use std::fmt::Debug;
    use std::ops::DerefMut;
    use std::sync::Arc;
    use tokio::spawn;
    use tokio::sync::Mutex;
    use uuid::fmt::Urn;
    use uuid::Uuid;
    ///RPC储蓄执行器
    pub struct RPCExecutor<'life, RE: Serialize + Deserialize<'life>, GX: Sized + Send + Sync>(
        pub BTreeMap<&'life str, Function<'life, RE, anyhow::Result<GX>>>,
    );

    impl<RE, GX> RPCExecutor<'static, RE, GX>
    where
        RE: Serialize + Deserialize<'static>,
        GX: Sized + Send + Sync,
    {
        pub async fn run_nc(&self, arg: RE, name: &str) -> anyhow::Result<Option<GX>>
        where
            RE: Send + Sync + 'static,
            GX: Debug + 'static,
        {
            Ok(if let Some(sv) = self.0.get(name) {
                let (a, b) = kanal::bounded_async(0);
                let xe = sv.start.clone();
                spawn(async move {
                    a.send(match xe.lock().await.deref_mut() {
                        FutureEx::AsyncTrait(e) => e.await?,
                        FutureEx::AsyncFnTrait(e) => e(arg).await?,
                        FutureEx::FnTrait(e) => e(arg)?,
                    })
                    .await?;
                    Ok(()) as anyhow::Result<()>
                });
                Some(b.recv().await?)
            } else {
                None
            })
        }
    }
    impl<'life> From<Database<'life>> for RPCExecutor<'life, Args<'life>, Args<'life>> {
        fn from(value: Database<'life>) -> Self {
            let mut bt = BTreeMap::new();
            value.0.into_iter().for_each(|e| {
                bt.insert(e.name, Function::from(e.function));
            });
            RPCExecutor(bt)
        }
    }
    impl<'life, RE, GX> From<Vec<(&'life str, Function<'life, RE, anyhow::Result<GX>>)>>
        for RPCExecutor<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn from(value: Vec<(&'life str, Function<'life, RE, anyhow::Result<GX>>)>) -> Self {
            let mut er = BTreeMap::new();
            value.into_iter().for_each(|(a, b)| {
                er.insert(a, b);
            });
            RPCExecutor(er)
        }
    }
    impl<'life, RE, GX, const NP: usize>
        From<OmegaIterator<(&'life str, Function<'life, RE, anyhow::Result<GX>>), NP>>
        for RPCExecutor<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send + Sync,
    {
        fn from(
            value: OmegaIterator<(&'life str, Function<'life, RE, anyhow::Result<GX>>), NP>,
        ) -> Self {
            let mut net = BTreeMap::new();
            value.into_iter().for_each(|(a, b)| {
                net.insert(a, b);
            });
            RPCExecutor(net)
        }
    }
    ///函数
    pub struct Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        ///名称
        id: String,
        ///函数
        pub start: Arc<Mutex<FutureEx<'life, RE, GX>>>,
    }

    impl<RE, GX> Function<'static, RE, GX>
    where
        RE: Serialize + Deserialize<'static>,
        GX: Sized + Send,
    {
        pub async fn run_async(&mut self, arg: RE) -> anyhow::Result<GX>
        where
            RE: Send + 'static,
            GX: 'static,
        {
            let er = self.start.clone();
            let (a, b) = kanal::bounded_async(0);
            spawn(async move {
                a.send(er.lock().await.run_sync(arg).await).await?;
                Ok(()) as anyhow::Result<()>
            });
            Ok(b.recv().await?)
        }
    }
    impl<'life, RE, GX> From<Box<dyn FnMut(RE) -> GX + Send + Sync + 'life>> for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn from(value: Box<dyn FnMut(RE) -> GX + Send + Sync + 'life>) -> Self {
            Function {
                id: Urn::from_uuid(Uuid::new_v4()).to_string(),
                start: Arc::new(Mutex::new(FutureEx::FnTrait(value))),
            }
        }
    }
    impl<'life, RE, GX> From<Box<dyn FnMut(RE) -> Async<'life, GX> + Send + Sync + 'life>>
        for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn from(value: Box<dyn FnMut(RE) -> Async<'life, GX> + Send + Sync + 'life>) -> Self {
            Function {
                id: Urn::from_uuid(Uuid::new_v4()).to_string(),
                start: Arc::new(Mutex::new(FutureEx::AsyncFnTrait(value))),
            }
        }
    }
    impl<'life, RE, GX> From<Async<'life, GX>> for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn from(value: Async<'life, GX>) -> Self {
            Function {
                id: Urn::from_uuid(Uuid::new_v4()).to_string(),
                start: Arc::new(Mutex::new(FutureEx::AsyncTrait(value))),
            }
        }
    }
    impl<'life, RE, GX> From<FutureEx<'life, RE, GX>> for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn from(value: FutureEx<'life, RE, GX>) -> Self {
            Function {
                id: Urn::from_uuid(Uuid::new_v4()).to_string(),
                start: Arc::new(Mutex::new(value)),
            }
        }
    }
    impl<'life, RE, GX> Hash for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.id.hash(state)
        }
    }
    ///排序
    impl<'life, RE, GX> Ord for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            self.id.cmp(&other.id)
        }
    }
    ///a>=b
    impl<'life, RE, GX> PartialOrd for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.id.partial_cmp(&other.id)
        }
    }
    ///a==a
    impl<'life, RE, GX> Eq for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
    }
    ///a==b
    impl<'life, RE, GX> PartialEq for Function<'life, RE, GX>
    where
        RE: Serialize + Deserialize<'life>,
        GX: Sized + Send,
    {
        fn eq(&self, other: &Self) -> bool {
            self.id.eq(&other.id)
        }
    }
}
pub mod client {
    use super::*;
    use crate::rpc_ie::parameter::Args;
    use std::cmp::Ordering;
    ///请求
    pub struct Request<'life>(pub HashSet<ApplyFor<'life>>);
    impl<'life> From<Database<'life>> for Request<'life> {
        fn from(value: Database<'life>) -> Self {
            let mut et = HashSet::new();
            value.0.into_iter().for_each(|e| {
                et.insert(ApplyFor {
                    subscription: e.name,
                    function: e.args,
                });
            });
            Request(et)
        }
    }
    impl<'life, const GP: usize> From<OmegaIterator<ApplyFor<'life>, GP>> for Request<'life> {
        fn from(value: OmegaIterator<ApplyFor<'life>, GP>) -> Self {
            let mut et = HashSet::new();
            value.0.into_iter().for_each(|x| {
                et.insert(x);
            });
            Request(et)
        }
    }
    ///请求
    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct ApplyFor<'life> {
        ///订阅内容
        pub subscription: &'life str,
        ///函数
        pub function: Args<'life>,
    }

    impl<'life> Ord for ApplyFor<'life> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.subscription.cmp(other.subscription)
        }
    }
    impl<'life> PartialOrd for ApplyFor<'life> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.subscription.partial_cmp(other.subscription)
        }
    }
    impl<'life> Eq for ApplyFor<'life> {}
    impl<'life> PartialEq for ApplyFor<'life> {
        fn eq(&self, other: &Self) -> bool {
            self.subscription.eq(other.subscription)
        }
    }
    impl<'life> From<(&'life str, Args<'life>)> for ApplyFor<'life> {
        fn from(value: (&'life str, Args<'life>)) -> Self {
            ApplyFor {
                subscription: value.0,
                function: value.1,
            }
        }
    }
    impl<'life> Hash for ApplyFor<'life> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.subscription.hash(state)
        }
    }
}
