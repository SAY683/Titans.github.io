use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::element::{Local, Logs, Node, Storage};

///设置文件
pub const SETTINGS_FILE: &str = "./Conf/Settings.xml";

pub trait InstallSrc {
	///#构建设置
	fn build_settings() -> anyhow::Result<Install> {
		let mut x = String::new();
		BufReader::new(File::open(SETTINGS_FILE)?).read_to_string(&mut x)?;
		Ok(quick_xml::de::from_str(&x)?)
	}
}

///#核心设置
#[derive(Debug, Serialize, Deserialize)]
pub struct Install {
	///#数据
	pub data: Data,
	pub drive: Drive,
}

///#数据
#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
	///保存路径
	pub storage: Storage,
	///日志
	pub logs: Logs,
}

///#驱动
#[derive(Debug, Serialize, Deserialize)]
pub struct Drive {
	///主节点
	pub master: Local,
	///节点
	#[serde(rename = "node", default)]
	pub slave: Vec<Node>,
}

pub mod element {
	use super::*;
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Logs {
		///路径
		pub path: PathBuf,
		///节点logs
		pub node_logs: PathBuf,
		///开启
		pub enabled: bool,
		///过期日期
		pub expiration_day: i64,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Local {
		///路径
		pub host: String,
		///集群开启
		pub cluster_enabled: bool,
		///发送构建
		pub scp_build: bool,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Node {
		///节点名称
		pub node_name: String,
		///节点路径
		pub host: String,
		///名称
		pub ssh_user: String,
		///密码
		pub ssh_password: String,
		///过期日期
		pub ssh_private_key_path: Option<PathBuf>,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Storage {
		///本地数据
		pub local_data: PathBuf,
		///节点数据
		pub node_data: PathBuf,
		///节点服务文件
		pub node_service_path: PathBuf,
	}
}

impl InstallSrc for Install {}