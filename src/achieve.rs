use crate::PUT_UP;

pub async fn achieve() -> anyhow::Result<()> {
	logs_is();
	Ok(())
}

fn logs_is() {
	let x = &PUT_UP.get().unwrap().data.logs;
	if x.enabled {
		ftlog::logger().flush();
	}
}