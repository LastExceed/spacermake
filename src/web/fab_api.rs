use std::sync::Arc;

use anyhow::anyhow;
use capnp_rpc::rpc_twoparty_capnp::Side;
use futures::io::{BufReader, BufWriter};
use futures::{AsyncReadExt, FutureExt};
use itertools::Itertools;
use pollster::FutureExt as _;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;
use async_native_tls::TlsConnector;
use tap::{Tap, Pipe};
use capnp_rpc::*;
use tokio::task;
use crate::my_config::MyConfig;
use crate::schema::machine_capnp::machine::MachineState;
use crate::schema::*;

pub mod object;

use object::*;

type Bootstrap = connection_capnp::bootstrap::Client;
type MachineSystemInfo = machinesystem_capnp::machine_system::info::Client;
type AuthRespWhich<A0, A1, A2> = authenticationsystem_capnp::response::Which<A0, A1, A2>;
type OptionalWhich<A0> = general_capnp::optional::Which<A0>;

pub async fn get_resources(username: &str, password: &str, to_toggle: Option<&str>, config: &Arc<MyConfig>) -> anyhow::Result<Vec<Machine>> {
    let username = username.to_owned();
    let password = password.to_owned();
    let to_toggle = to_toggle.map(str::to_owned);
    let config = Arc::clone(config);
    task::spawn_blocking(move || {
        task::LocalSet
        ::new()
        .run_until(do_rpc(&username, &password, to_toggle.as_ref().map(|s| s.as_str()), &config))
        .block_on()
    })
    .await?
}

async fn do_rpc(username: &str, password: &str, to_toggle: Option<&str>, config: &MyConfig) -> anyhow::Result<Vec<Machine>> {
    let mut rpc_system = connect_rpc(&config).await?;
    let bootstrap = rpc_system.bootstrap::<Bootstrap>(Side::Server);
    task::spawn_local(rpc_system);
    
    let machine_system_info = try_api_login(&bootstrap, username, password).await?;

    if let Some(target) = to_toggle {
        toggle_machine(&machine_system_info, target).await?;
    }

    get_machines(&machine_system_info).await
}

async fn connect_rpc(config: &MyConfig) -> anyhow::Result<RpcSystem<Side>> {
    let stream = TcpStream::connect((config.fabaccess_host.as_str(), config.fabaccess_port)).await?;
    stream.set_nodelay(true)?;
    
    TlsConnector
    ::new()
    .danger_accept_invalid_certs(true)
    .connect(&config.fabaccess_host, stream)
    .await?
    .pipe(TokioAsyncReadCompatExt::compat)
    .split()
    .pipe(|(reader, writer)|
        twoparty::VatNetwork::new(
            BufReader::new(reader),
            BufWriter::new(writer),
            Side::Client,
            Default::default(),
        )
    )
    .pipe(Box::new)
    .pipe(|network| RpcSystem::new(network, None))
    .pipe(Ok)
}

async fn try_api_login(bootstrap: &Bootstrap, username: &str, password: &str) -> anyhow::Result<MachineSystemInfo> {
    bootstrap
    .create_session_request()
    .tap_mut(|req| req.get().set_mechanism("PLAIN"))
    .send()
    .promise
    .await?
    .get()?
    .get_authentication()?
    .step_request()
    .tap_mut(|req| req.get().set_data(format!("\0{username}\0{password}").as_bytes()))
    .send()
    .promise
    .await?
    .get()?
    .which()?
    .pipe(|which| match which {
        AuthRespWhich::Failed(a0)     => Err(anyhow!("{a0:?}")),
        AuthRespWhich::Challenge(a1)  => Err(anyhow!("{a1:?}")),
        AuthRespWhich::Successful(a2) => Ok(a2),
    })?
    .get_session()?
    .get_machine_system()?
    .get_info()?
    .pipe(Ok)
}

async fn get_machines(machine_system_info: &MachineSystemInfo) -> anyhow::Result<Vec<Machine>> {
    machine_system_info
    .get_machine_list_request()
    .send()
    .promise
    .await?
    .get()?
    .get_machine_list()?
    .into_iter()
    .map(Machine::try_from)
    .try_collect()
    .map_err(capnp::Error::into)
}

async fn toggle_machine(machine_system_info: &MachineSystemInfo, target: &str) -> anyhow::Result<()> {
    machine_system_info
    .get_machine_u_r_n_request()
    .tap_mut(|x| x.get().set_urn(target))
    .send()
    .promise
    .await?
    .get()?
    .which()?
    .pipe(|which| match which {
        OptionalWhich::Just(machine) => Ok(machine),
        OptionalWhich::Nothing(()) => Err(anyhow!("machine not found")),
    })??
    .pipe(async |machine| {
        if machine.has_inuse() {
            machine.get_inuse()?.give_back_request().send().promise.await?.get().map(|_| ())
        } else if machine.get_state()? == MachineState::InUse {
            machine.get_manage()?.force_free_request().send().promise.await?.get().map(|_| ())
        } else {
            machine.get_use()?.use_request().send().promise.await?.get().map(|_| ())
        }
    })
    .await?
    .pipe(Ok)
}