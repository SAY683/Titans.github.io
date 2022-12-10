///#执行阶段
pub mod beginning_of {}

///#初始阶段
pub mod initial_state {
	use ftlog::{FtLogFormatter, info, LevelFilter};
	use ftlog::appender::{Duration, FileAppender, Period};
	use Gui_src::{Colour, Information, View};
	use Install_src::{Install, InstallSrc};
	use Network_src::ssh_sc::SshSc;
	use crate::PUT_UP;
	
	pub async fn initial() -> anyhow::Result<()> {
		Colour::error_display("construct", construct());
		Colour::error_display("logs_to", logs_to());
		if PUT_UP.get().unwrap().drive.master.cluster_enabled {
			Colour::error_display("node_build", node_build());
		}
		Ok(())
	}
	
	///#构建
	fn construct() -> anyhow::Result<()> {
		PUT_UP.get_or_init(|| Colour::error_display("PUT_UP", Install::build_settings()));
		Ok(())
	}
	
	///#日志
	fn logs_to() -> anyhow::Result<()> {
		let x = &Colour::operation("PUT_UP", PUT_UP.get()).data.logs;
		if x.enabled {
			ftlog::builder()
				.format(FtLogFormatter)
				.max_log_level(LevelFilter::Info)
				.root(FileAppender::rotate_with_expire(
					x.path.as_os_str(),
					Period::Day,
					Duration::days(x.expiration_day),
				)).build()?.init()?;
		}
		Ok(())
	}
	
	///#节点构建
	fn node_build() -> anyhow::Result<()> {
		for i in PUT_UP.get().unwrap().drive.slave.iter() {
			i.shell_one(format!("mkdir {}", PUT_UP.get().unwrap().data.storage.node_data.to_str().unwrap()).as_str()).unwrap_or_else(|r| {
				println!("{}", Colour::Debug.table(Information {
					list: ["PossibleError"],
					data: [[format!("{r:?}").as_str()], ["ConfigurationError"]],
				}));
				String::new()
			});
			info!("{}-->{}Node_Build<+OK+>",i.node_name,i.host);
		}
		Ok(())
	}
}