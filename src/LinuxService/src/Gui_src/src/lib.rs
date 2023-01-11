use std::fmt::{Debug, Display};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use ftlog::{debug, error, info};

///#风格
pub enum Colour {
	///错误
	Error,
	///输出
	Output,
	///命令
	Order,
	///监控
	Monitoring,
	///函数
	Function,
	//
	Debug,
}

///#数据
pub struct Information<'life, const GC: usize, const FG: usize> {
	///列表
	pub list: [&'life str; GC],
	///数据
	pub data: [[&'life str; GC]; FG],
}

///#数据列表
pub struct Information2<'life> {
	///列表
	pub list: Vec<&'life str>,
	///数据
	pub data: Vec<Vec<String>>,
}

///#显示
pub trait View {
	fn table<const GC: usize, const FG: usize>(&self, _: Information<GC, FG>) -> Table;
	///#fn error_display<G: Sized>(r: &str, e: anyhow::Result<G>)->G
	fn error_display<G: Sized>(r: &str, e: anyhow::Result<G>) -> G {
		e.unwrap_or_else(|x| {
			error!("{}",x);
			panic!(
				"{}",
				Colour::Error.table(Information {
					list: [r],
					data: [[format!("{x}").as_str()]],
				})
			)
		})
	}
	///#fn error_display_ty<G: Sized>(r: &str, e: anyhow::Result<G>, v: fn() -> G) -> G
	fn error_display_ty<G: Sized>(r: &str, e: anyhow::Result<G>, v: fn() -> G) -> G {
		e.unwrap_or_else(|x| {
			debug!("{}",x);
			eprintln!(
				"{}",
				Colour::Error.table(Information {
					list: [r],
					data: [[format!("{x}").as_str()]],
				})
			);
			v()
		})
	}
	///fn error_display_ot<G: Sized, T: Debug + Display>(r: &str, e: Result<G, T>) -> G
	fn error_display_ot<G: Sized, T: Debug + Display>(r: &str, e: Result<G, T>) -> G {
		e.unwrap_or_else(|x| {
			error!("{}",x);
			panic!(
				"{}",
				Colour::Error.table(Information {
					list: [r],
					data: [[format!("{x}").as_str()]],
				})
			);
		})
	}
	///fn error_display_ty_ot<G: Sized, T: Debug + Display>(r: &str, e: Result<G, T>, v: fn() -> G) -> G
	fn error_display_ty_ot<G: Sized, T: Debug + Display>(r: &str, e: Result<G, T>, v: fn() -> G) -> G {
		e.unwrap_or_else(|x| {
			debug!("{}",x);
			println!(
				"{}",
				Colour::Error.table(Information {
					list: [r],
					data: [[format!("{x}").as_str()]],
				})
			);
			v()
		})
	}
	///#fn operation<G: Sized>(r: &str, e: Option<G>) -> G
	fn operation<G: Sized>(r: &str, e: Option<G>) -> G {
		e.unwrap_or_else(|| {
			error!("{}",r);
			panic!(
				"{}",
				Colour::Error.table(Information {
					list: [r],
					data: [["None"]],
				})
			)
		})
	}
	fn table2(&self, _: Information2) -> Table;
	///默认
	fn table_default() -> Table {
		let mut table = Table::new();
		table
			.load_preset(UTF8_FULL)
			.apply_modifier(UTF8_ROUND_CORNERS)
			.set_content_arrangement(ContentArrangement::DynamicFullWidth)
			.set_width(40);
		table
	}
}

impl View for Colour {
	fn table<const GC: usize, const FG: usize>(&self, e: Information<GC, FG>) -> Table {
		let i = Colour::view(self);
		let mut table = Colour::table_default();
		table.set_header(
			e.list
				.map(|x| Cell::new(x).add_attribute(i.text).fg(i.frames))
				.into_iter()
				.collect::<Vec<_>>(),
		);
		e.data.into_iter().for_each(|x| {
			table.add_row(
				x.map(|x| Cell::new(x).add_attribute(i.text).fg(i.frames))
					.into_iter()
					.collect::<Vec<_>>(),
			);
		});
		table
	}
	
	fn table2(&self, e: Information2) -> Table {
		let i = Colour::view(self);
		let mut table = Colour::table_default();
		table.set_header(e.list.into_iter().map(|x| { Cell::new(x).add_attribute(i.text).fg(i.frames) }).collect::<Vec<_>>());
		e.data.into_iter().for_each(|x| {
			table.add_row(
				x.into_iter().map(|x| Cell::new(x).add_attribute(i.text).fg(i.frames))
					.into_iter()
					.collect::<Vec<_>>(),
			);
		});
		table
	}
}

///#画面数据
pub struct Frames {
	//文本
	text: Attribute,
	//单元格前景色
	frames: Color,
}

impl Colour {
	pub fn logs_is(self, r: &str) -> Self {
		match self {
			Colour::Error => { error!("{}",r) }
			Colour::Monitoring => { info!("{}",r); }
			Colour::Debug => { debug!("{}",r) }
			_ => {}
		}
		self
	}
	fn view(&self) -> Frames {
		match self {
			Colour::Error => Frames {
				text: Attribute::Italic,
				frames: Color::DarkRed,
			},
			Colour::Output => Frames {
				text: Attribute::Bold,
				frames: Color::DarkGreen,
			},
			Colour::Order => Frames {
				text: Attribute::RapidBlink,
				frames: Color::DarkYellow,
			},
			Colour::Monitoring => Frames {
				text: Attribute::Underlined,
				frames: Color::DarkCyan,
			},
			Colour::Function => Frames {
				text: Attribute::Reverse,
				frames: Color::DarkGrey,
			},
			Colour::Debug => Frames {
				text: Attribute::Reverse,
				frames: Color::Black,
			}
		}
	}
}