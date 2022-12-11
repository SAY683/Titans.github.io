#![warn(non_snake_case)]

mod beginning;
mod achieve;

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use once_cell::sync::OnceCell;
use tokio::{main, spawn};
use tokio::task::JoinHandle;
use Gui_src::Colour;
use crate::beginning::initial_state::initial;
use Install_src::Install;
use crate::achieve::achieve;
use crate::execute::Execution;

#[main]
pub async fn main() -> anyhow::Result<()> {
	let start = Arc::new((Mutex::new(false), Condvar::new()));
	let start_node = start.clone();
	Execution::from(([spawn(async move {
		initial(start).await?;
		achieve().await?;
		Ok(())
	}), spawn(async move {
		let (x, y) = &*start_node;
		if *y.wait(x.lock().unwrap()).unwrap() {}
		Ok(())
	})], "Master")).run_async().await?;
	Ok(())
}

///#执行异步池
pub mod execute {
	use super::*;
	use core::iter::IntoIterator;
	use std::array;
	use ftlog::{info};
	use Gui_src::{Information, View};
	
	
	///#执行
	pub struct Execution<'life, const GX: usize> {
		pub service: [JoinHandle<anyhow::Result<()>>; GX],
		pub name: &'life str,
	}
	
	impl<'life, const GX: usize> From<[JoinHandle<anyhow::Result<()>>; GX]> for Execution<'life, GX> {
		fn from(value: [JoinHandle<anyhow::Result<()>>; GX]) -> Self {
			Execution { service: value, name: "default" }
		}
	}
	
	impl<'life, const GX: usize> From<([JoinHandle<anyhow::Result<()>>; GX], &'life str)> for Execution<'life, GX> {
		fn from(value: ([JoinHandle<anyhow::Result<()>>; GX], &'life str)) -> Self {
			Execution { service: value.0, name: value.1 }
		}
	}
	
	impl<'life, const GX: usize> IntoIterator for Execution<'life, GX> {
		type Item = JoinHandle<anyhow::Result<()>>;
		type IntoIter = array::IntoIter<tokio::task::JoinHandle<std::result::Result<(), anyhow::Error>>, GX>;
		fn into_iter(self) -> Self::IntoIter {
			self.service.into_iter()
		}
	}
	
	impl<'life, const GX: usize> Execution<'life, GX> {
		pub fn run(self) {
			if THREAD_DISPLAY.load(Ordering::SeqCst) {
				println!("{}", Colour::Monitoring.table(Information { list: ["Thread"], data: [[self.name]] }))
			} else {
				info!("<+>[{}]<+>",self.name);
			}
			self.service.map(|run| {
				run
			});
		}
		pub async fn run_async(self) -> anyhow::Result<()> {
			if THREAD_DISPLAY.load(Ordering::SeqCst) {
				println!("{}", Colour::Monitoring.table(Information { list: ["Async Thread"], data: [[self.name]] }));
			} else {
				info!("<+>[{}]<+>",self.name);
			}
			for i in self.service {
				Colour::error_display(self.name, i.await?);
			}
			Ok(())
		}
	}
}

///#设置
pub static PUT_UP: OnceCell<Install> = OnceCell::new();
///线程名称显示
pub static THREAD_DISPLAY: AtomicBool = AtomicBool::new(false);
///线程计数
pub static MASTER_ID: AtomicI64 = AtomicI64::new(0);
///节点安装包
pub const SLAVE_PKG: &str = "./TitansService";