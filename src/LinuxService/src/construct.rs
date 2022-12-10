use Gui_src::{Colour, Information, View};
use crate::{INSTALL_SRC, LOCAL_IP};

///构建
pub async fn build() -> anyhow::Result<()> {
	println!("{}", Colour::Output.table(Information {
		list: ["local_socket", "master_socket"],
		data: [[LOCAL_IP.as_ref().unwrap().as_str(), INSTALL_SRC.load().drive.master.host.as_str()]],
	}));
	Ok(())
}