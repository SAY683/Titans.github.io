use std::net::{TcpStream};
use std::path::{Path};
use hashbrown::HashMap;
use jsonrpsee::tracing::info;
use ssh_rs::{LocalSession, SessionBroker};
use ssh_rs::ssh::create_session;
use Install_src::element::Node;

pub trait SshSc {
	///#ssh 多线程
	fn get_shh(&self) -> anyhow::Result<SessionBroker>;
	///#ssh 单线程
	fn get_shh_local(&self) -> anyhow::Result<LocalSession<TcpStream>>;
	///#普通shell
	fn shell<'life>(&self, _: Vec<&'life str>) -> anyhow::Result<HashMap<&'life str, String>>;
	///#单次单线程shell
	fn shell_one(&self, _: &str) -> anyhow::Result<String>;
	///scp 多线程
	fn scp(&self, _: &Path, _: &Path, _: bool) -> anyhow::Result<()>;
}

impl SshSc for Node {
	fn get_shh(&self) -> anyhow::Result<SessionBroker> {
		Ok(if let Some(x) = &self.ssh_private_key_path {
			create_session()
				.username(&self.ssh_user)
				.password(&self.ssh_password)
				.timeout(1000)
				.private_key_path(x)
				.connect(&self.host)?.run_backend()
		} else {
			create_session()
				.username(&self.ssh_user)
				.password(&self.ssh_password)
				.timeout(1000)
				.connect(&self.host)?.run_backend()
		})
	}
	
	fn get_shh_local(&self) -> anyhow::Result<LocalSession<TcpStream>> {
		Ok(if let Some(x) = &self.ssh_private_key_path {
			create_session()
				.username(&self.ssh_user)
				.password(&self.ssh_password)
				.timeout(1000)
				.private_key_path(x)
				.connect(&self.host)?.run_local()
		} else {
			create_session()
				.username(&self.ssh_user)
				.password(&self.ssh_password)
				.timeout(1000)
				.connect(&self.host)?.run_local()
		})
	}
	
	fn shell<'life>(&self, e: Vec<&'life str>) -> anyhow::Result<HashMap<&'life str, String>> {
		let mut ec = self.get_shh()?;
		let mut fg = HashMap::new();
		for i in e {
			let t = ec.open_exec()?;
			t.send_command(i)?;
			fg.insert(i, String::from_utf8(t.get_result()?)?);
		}
		ec.close();
		Ok(fg)
	}
	
	fn shell_one(&self, rc: &str) -> anyhow::Result<String> {
		let mut x = self.get_shh_local()?;
		let r = x.open_exec()?;
		let rt = String::from_utf8(r.send_command(rc)?)?;
		x.close();
		Ok(rt)
	}
	fn scp(&self, r: &Path, j: &Path, t: bool) -> anyhow::Result<()> {
		let mut z = self.get_shh()?;
		let mut g = z.open_scp()?;
		match t {
			true => {
				g.upload(r, j)?;
				info!("{r:?}--Download->{j:?}");
			}
			false => {
				g.start_download(r, j)?;
				info!("{j:?}--Download->{r:?}");
				g.end_download()?;
			}
		};
		z.close();
		Ok(())
	}
}