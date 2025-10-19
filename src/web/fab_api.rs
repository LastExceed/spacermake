use anyhow::anyhow;
use capnp_rpc::rpc_twoparty_capnp::Side;
use futures::channel::mpsc::{self, Receiver, Sender};
use futures::io::{BufReader, BufWriter};
use futures::lock::Mutex;
use futures::{AsyncReadExt, SinkExt};
use itertools::Itertools;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;
use async_native_tls::TlsConnector;
use tap::{Tap, Pipe};
use capnp_rpc::*;
use futures::StreamExt;
use tokio::task::LocalSet;
use crate::schema::machine_capnp::machine::MachineState;
use crate::schema::*;

pub mod object;

use object::*;

type Bootstrap = connection_capnp::bootstrap::Client;
type MachineSystemInfo = machinesystem_capnp::machine_system::info::Client;
type AuthRespWhich<A0, A1, A2> = authenticationsystem_capnp::response::Which<A0, A1, A2>;
type OptionalWhich<A0> = general_capnp::optional::Which<A0>;

type ChannelResponse = Result<Vec<Machine>, anyhow::Error>;

pub struct ChannelRequest {
    pub username: String,
    pub password: String,
    pub to_toggle: Option<String>
}

pub struct FrontDesk {
    sender: Mutex<Sender<ChannelRequest>>,
    receiver: Mutex<Receiver<ChannelResponse>>
}

impl FrontDesk {
    fn new(sender: Sender<ChannelRequest>, receiver: Receiver<ChannelResponse>) -> Self {
        Self {
            sender: Mutex::new(sender),
            receiver: Mutex::new(receiver)
        }
    }

    pub async fn exchange(&self, username: String, password: String, to_toggle: Option<String>) -> ChannelResponse {
        let req = ChannelRequest {
            username,
            password,
            to_toggle
        };

        let mut send_lock = self.sender.lock().await;
        send_lock.send(req).await.unwrap();
        let mut recv_lock = self.receiver.lock().await;
        drop(send_lock);
        recv_lock.next().await.unwrap()
    }
}

pub async fn start_local(local_set: &LocalSet) -> FrontDesk {
    let (req_snd, mut req_recv) = mpsc::channel(1);
    let (mut rsp_snd, rsp_recv) = mpsc::channel(1);

    let mut rpc_system = connect_rpc().await;
    let bootstrap = rpc_system.bootstrap::<Bootstrap>(Side::Server);

    local_set.spawn_local(rpc_system);
    local_set.spawn_local(async move {
        #[expect(clippy::infinite_loop, reason = "intended")]
        loop {
            let request = req_recv.next().await.unwrap();
            let response = handle_request(&bootstrap, request).await;
            rsp_snd.send(response).await.unwrap();
        }
    });
    
    FrontDesk::new(req_snd, rsp_recv)
}

async fn handle_request(bootstrap: &Bootstrap, ChannelRequest { username, password, to_toggle }: ChannelRequest) -> anyhow::Result<Vec<Machine>> {
    let machine_system_info = try_api_login(bootstrap, &username, &password).await?;

    if let Some(target) = to_toggle {
        toggle_machine(&machine_system_info, target).await?;
    }
    
    get_machines(&machine_system_info).await
}

async fn connect_rpc() -> RpcSystem<Side> {
    let tls_connector = TlsConnector::new().danger_accept_invalid_certs(true);

    TcpStream::connect("test.fab-access.org:59661")
    .await
    .unwrap()
    .tap(|stream|stream.set_nodelay(true).unwrap())
    .pipe(|stream| tls_connector.connect("test.fab-access.org", stream))
    .await
    .unwrap()
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
}

pub async fn try_api_login(bootstrap: &Bootstrap, username: &str, password: &str) -> anyhow::Result<MachineSystemInfo> {
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

pub async fn get_machines(machine_system_info: &MachineSystemInfo) -> anyhow::Result<Vec<Machine>> {
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

async fn toggle_machine(machine_system_info: &MachineSystemInfo, target: String) -> anyhow::Result<()> {
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