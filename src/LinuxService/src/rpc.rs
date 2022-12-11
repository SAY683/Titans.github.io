use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use jsonrpsee::server::{Server, ServerBuilder};
use jsonrpsee::types::Params;
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use jsonrpsee_core::params::ArrayParams;
use jsonrpsee_core::rpc_params;
use jsonrpsee_core::server::rpc_module::RpcModule;

#[derive(Debug, Serialize, Deserialize)]
pub struct Bit {
	pub base: String,
}

#[async_trait]
pub trait RPC<'life>: Serialize + Deserialize<'life> + Sync + Send + Sized {
	///#服务
	async fn server(ulr: &str) -> anyhow::Result<Server> {
		Ok(ServerBuilder::default().build(ulr).await?)
	}
	///#服务句柄
	fn execute<GX: Sized + Send + Sync>(e: GX) -> anyhow::Result<RpcModule<GX>> {
		Ok(RpcModule::new(e))
	}
	///#链接
	async fn connect(ulr: &str) -> anyhow::Result<WsClient> {
		Ok(WsClientBuilder::default()
			.build(&format!("ws://{ulr}"))
			.await?)
	}
	///#转换
	///#[ArrayParams]
	fn args<Gx: Serialize + Deserialize<'life>>(e: &Gx) -> anyhow::Result<ArrayParams> {
		Ok(rpc_params!(serde_json::to_string(e)?))
	}
	///转换1 params
	fn analysis_params(e: Params) -> anyhow::Result<Bit> {
		Ok(serde_json::from_str::<Bit>(e.parse::<Bit>()?.base.as_str())?)
	}
	///转换2 解析
	fn analysis_resolve<SF: Serialize + Deserialize<'life> + Sync + Send + Sized>(
		bit: &'life str,
	) -> anyhow::Result<SF> {
		Ok(serde_json::from_str::<SF>(bit)?)
	}
	///转换3 bit
	fn analysis_bit<SF: Serialize + Deserialize<'life> + Sync + Send + Sized>(
		e: &SF,
	) -> anyhow::Result<Bit> {
		Ok(Bit {
			base: serde_json::to_string(e)?,
		})
	}
	///Serialize
	fn analysis(&self) -> anyhow::Result<Bit> {
		Ok(Bit { base: serde_json::to_string(self)? })
	}
	///Deserialize
	fn resolve(bit: &'life str) -> anyhow::Result<Self> {
		Ok(serde_json::from_str(bit)?)
	}
}