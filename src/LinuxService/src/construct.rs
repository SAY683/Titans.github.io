use std::net::SocketAddr;
use Gui_src::{Colour, Information, View};
use crate::{INSTALL_SRC, LOCAL_HOST, LOCAL_IP};

///构建
pub async fn build() -> anyhow::Result<()> {
	echo()?;
	master_ping(INSTALL_SRC.load().drive.master.host.parse()?).await?;
	Ok(())
}

fn echo() -> anyhow::Result<()> {
	let mut na = String::new();
	let e = LOCAL_IP.as_ref().unwrap();
	INSTALL_SRC.load().drive.slave.iter().for_each(|i| {
		if &i.host.parse::<SocketAddr>().unwrap().ip().to_string() == e {
			na = i.node_name.to_string();
		}
	});
	if na.is_empty() {
		na = "UnableToConfirmNodeName".to_string();
	}
	println!("{}", Colour::Output.table(Information {
		list: ["Local_Host", "Master_Host", "NodeName"],
		data: [[LOCAL_HOST.as_ref().unwrap().to_string().as_str(), INSTALL_SRC.load().drive.master.host.as_str(), &na]],
	}));
	Ok(())
}

async fn master_ping(r: SocketAddr) -> anyhow::Result<bool> {
	Ok(false)
}