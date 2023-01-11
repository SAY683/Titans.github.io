use crate::{INSTALL_SRC, LOCAL_HOST, LOCAL_IP, NODE, NODE_NAME};
use std::net::SocketAddr;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::Ordering;
use Gui_src::{Colour, Information, View};
use Install_src::{Install, InstallSrc};

///构建
pub async fn build(start_main: Arc<(Mutex<bool>, Condvar)>) -> anyhow::Result<()> {
	bach()?;
	echo()?;
	let (mutex, condvar) = &*start_main;
	*mutex.lock().unwrap() = true;
	condvar.notify_all();
	Ok(())
}

pub async fn start_link() -> anyhow::Result<()> {
	Ok(())
}

//启动脚本
fn bach() -> anyhow::Result<()> {
	let mut na = String::new();
	let e = LOCAL_IP.as_ref().unwrap();
	Install::build_settings()?
		.drive
		.slave
		.into_iter()
		.for_each(|i| {
			if &i.host.parse::<SocketAddr>().unwrap().ip().to_string() == e {
				na = i.node_name.to_string();
				NODE.get_or_init(|| i);
			}
		});
	if na.is_empty() {
		let r = INSTALL_SRC
			.load()
			.drive
			.master
			.host
			.parse::<SocketAddr>()?
			.ip()
			.to_string();
		if &r == e {
			na = "Master".to_string();
		} else {
			na = "UnableToConfirmNodeName".to_string();
		}
	}
	NODE_NAME.get_or_init(|| na);
	Ok(())
}

//显示
fn echo() -> anyhow::Result<()> {
	println!(
		"{}",
		Colour::Output.table(Information {
			list: ["Local_Host", "Master_Host", "NodeName"],
			data: [[
				LOCAL_HOST.as_ref().unwrap().to_string().as_str(),
				INSTALL_SRC.load().drive.master.host.as_str(),
				NODE_NAME.get().unwrap().as_str()
			]],
		})
	);
	Ok(())
}
