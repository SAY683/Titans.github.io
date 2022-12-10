pub mod rpc;
mod construct;

use lazy_static::lazy_static;
use tokio::main;
use std::net::{UdpSocket, SocketAddr};
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use Install_src::{Install, InstallSrc};
use crate::construct::build;

#[main]
//#[cfg(target_os = "linux")]
async fn main() -> anyhow::Result<()> {
	build().await?;
	Ok(())
}
lazy_static! {
    pub static ref LOCAL_IP: anyhow::Result<String> = {
        let x = UdpSocket::bind("0.0.0.0:0")?;
        x.connect("8.8.8.8:80")?;
        Ok(x.local_addr()?.ip().to_string())
    };
    pub static ref LOCAL_HOST: anyhow::Result<SocketAddr> = {
        let x = UdpSocket::bind("0.0.0.0:0")?;
        x.connect("8.8.8.8:80")?;
        Ok(x.local_addr()?)
    };
    pub static ref LOCAL_PORT: anyhow::Result<String> = {
        let x = UdpSocket::bind("0.0.0.0:0")?;
        x.connect("8.8.8.8:80")?;
        Ok(x.local_addr()?.port().to_string())
    };
}
pub static INSTALL_SRC: Lazy<ArcSwap<Install>> = Lazy::new(|| {
	ArcSwap::from_pointee(Install::build_settings().unwrap())
});
