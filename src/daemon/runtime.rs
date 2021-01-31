// MyCitadel: node, wallet library & command-line tool
// Written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@mycitadel.io>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use std::{thread, time};

use internet2::zmqsocket::{self, ZmqType};
use internet2::{
    session, CreateUnmarshaller, PlainTranscoder, Session, TypedEnum,
    Unmarshall, Unmarshaller,
};
use lnpbp::strict_encoding::StrictDecode;
use microservices::node::TryService;
use microservices::FileFormat;
use rgb20::Asset;
use rgb_node::rpc::reply::SyncFormat;
use rgb_node::util::ToBech32Data;

use super::Config;
use crate::rpc::{Reply, Request};
use crate::storage::{self, Driver, FileDriver};
use crate::Error;

pub fn run(config: Config) -> Result<(), Error> {
    let runtime = Runtime::init(config)?;

    runtime.run_or_panic("mycitadeld");

    Ok(())
}

pub struct Runtime {
    /// Original configuration object
    config: Config,

    /// Stored sessions
    session_rpc: session::Raw<PlainTranscoder, zmqsocket::Connection>,

    /// Secure key vault
    storage: FileDriver,

    /// Unmarshaller instance used for parsing RPC request
    unmarshaller: Unmarshaller<Request>,

    /// RGB20 (fungibled) daemon client
    rgb20_client: rgb_node::i9n::Runtime,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, Error> {
        debug!("Initializing data storage {:?}", config.storage_conf());
        let storage = FileDriver::with(config.storage_conf())?;

        debug!("Opening ZMQ socket {}", config.rpc_endpoint);
        let session_rpc = session::Raw::with_zmq_unencrypted(
            ZmqType::Rep,
            &config.rpc_endpoint,
            None,
            None,
        )?;

        let rgb20_client =
            rgb_node::i9n::Runtime::init(rgb_node::i9n::Config {
                verbose: config.verbose,
                data_dir: config.data_dir.clone().to_string_lossy().to_string(),
                electrum_server: config.electrum_server.clone(),
                stash_rpc_endpoint: s!("inproc://stash.rpc"),
                stash_pub_endpoint: s!("inproc://stash.pub"),
                fungible_pub_endpoint: s!("inproc://fungible.pub"),
                contract_endpoints: map! {
                    rgb_node::rgbd::ContractName::Fungible => config.rgb20_endpoint.to_string()
                },
                network: config.chain.clone(),
                run_embedded: false,
            })
            .map_err(|err| Error::EmbeddedNodeError)?;

        thread::sleep(time::Duration::from_secs(1));

        Ok(Self {
            config,
            session_rpc,
            storage,
            rgb20_client,
            unmarshaller: Request::create_unmarshaller(),
        })
    }
}

impl TryService for Runtime {
    type ErrorType = Error;

    fn try_run_loop(mut self) -> Result<(), Self::ErrorType> {
        loop {
            match self.run() {
                Ok(_) => debug!("API request processing complete"),
                Err(err) => {
                    error!("Error processing API request: {}", err);
                    Err(err)?;
                }
            }
        }
    }
}

impl Runtime {
    fn run(&mut self) -> Result<(), Error> {
        trace!("Awaiting for ZMQ RPC requests...");
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.rpc_process(raw).unwrap_or_else(|err| err);
        trace!("Preparing ZMQ RPC reply: {:?}", reply);
        let data = reply.serialize();
        trace!(
            "Sending {} bytes back to the client over ZMQ RPC",
            data.len()
        );
        self.session_rpc.send_raw_message(&data)?;
        Ok(())
    }

    fn rpc_process(&mut self, raw: Vec<u8>) -> Result<Reply, Reply> {
        trace!(
            "Got {} bytes over ZMQ RPC: {}",
            raw.len(),
            raw.to_bech32data()
        );
        let message = (&*self.unmarshaller.unmarshall(&raw)?).clone();
        debug!(
            "Received ZMQ RPC request #{}: {}",
            message.get_type(),
            message
        );
        match message {
            Request::ListWallets => {
                self.storage.wallets().map(|list| Reply::Wallets(list))
            }
            Request::ListIdentities => self
                .storage
                .identities()
                .map(|list| Reply::Identities(list)),
            Request::ListAssets => self
                .rgb20_client
                .list_assets(FileFormat::StrictEncode)
                .map_err(|_| storage::Error::Remote)
                .and_then(|SyncFormat(_, data)| {
                    Vec::<Asset>::strict_deserialize(data)
                        .map_err(storage::Error::from)
                })
                .map(|assets| Reply::Assets(assets)),
            Request::AddWallet(contract) => {
                self.storage.add_wallet(contract).map(|_| Reply::Success)
            }
            Request::AddSigner(account) => {
                self.storage.add_signer(account).map(|_| Reply::Success)
            }
            Request::AddIdentity(identity) => {
                self.storage.add_identity(identity).map(|_| Reply::Success)
            }
            Request::AddAsset(genesis) => {
                unimplemented!()
            }
        }
        .map_err(Error::from)
        .map_err(Error::into)
    }
}