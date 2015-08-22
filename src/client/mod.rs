#![allow(deprecated)]

use ::error::Result;
use ::plugin_core::NewRpcRequest;
use mio::unix::UnixStream;
use mio;
use serde;
use std::path;
use std::sync::mpsc;

pub struct Client<'a> {
    event_loop_commands: mio::Sender<event_loop::Command>,
    rpc_loop_commands: mpsc::Sender<rpc_loop::Command>,
    rpc_loop_new_rpcs: mpsc::Sender<rpc_loop::NewRpc<'a>>,

    _rpc_loop_thread_join_guard: ::thread_scoped::JoinGuard<'a, ()>,
    _event_loop_thread_join_guard: ::thread_scoped::JoinGuard<'a, ()>,
}

impl<'a> Client<'a> {
    pub fn connect(socket_name: &path::Path) -> Result<Self> {
        let stream = try!(UnixStream::connect(&socket_name));

        let (commands_tx, commands_rx) = mpsc::channel();
        let (new_rpcs_tx, new_rpcs_rx) = mpsc::channel();
        let (event_loop_thread, event_loop_commands) = event_loop::spawn(stream, commands_tx.clone());

        Ok(Client {
            event_loop_commands: event_loop_commands.clone(),
            rpc_loop_commands: commands_tx,
            rpc_loop_new_rpcs: new_rpcs_tx,
            _rpc_loop_thread_join_guard: rpc_loop::spawn(commands_rx, new_rpcs_rx, event_loop_commands),
            _event_loop_thread_join_guard: event_loop_thread,
        })
    }

    pub fn new_rpc(&self, name: &str, rpc: Box<rpc::server::Rpc + 'a>) {
        // NOCOM(#sirver): what happens when this is already inserted? crash probably
        let mut new_rpc = self.call("core.new_rpc", &NewRpcRequest {
            priority: rpc.priority(),
            name: name.into(),
        });
        let success = new_rpc.wait().unwrap();
        // NOCOM(#sirver): report failure.

        self.rpc_loop_new_rpcs.send(rpc_loop::NewRpc::new(name.into(), rpc)).expect("NewRpc");
    }

    pub fn call<T: serde::Serialize>(&self, function: &str, args: &T) -> rpc::client::Context {
        rpc::client::Context::new(&self.commands_tx.clone(), function, args).unwrap()
    }

    pub fn clone(&self) -> ThinClient {
        ThinClient {
            commands: self.commands_tx.clone(),
        }
    }
}

impl<'a> Drop for Client<'a> {
    fn drop(&mut self) {
        // Either thread might have panicked at this point, so we can not rely on the sends to go
        // through. We just tell both (again) to Quit and hope they actually join.
        let _ = self.rpc_loop_commands.send(rpc_loop::Command::Quit);
        let _ = self.event_loop_commands.send(event_loop::Command::Quit);
    }
}

#[derive(Clone)]
pub struct ThinClient {
    commands: mpsc::Sender<rpc_loop::Command>,
}

// NOCOM(#sirver): figure out the difference between a Sender, an Context and come up with better
// names.
impl ThinClient {
    pub fn call<T: serde::Serialize>(&self, function: &str, args: &T) -> rpc::client::Context {
        rpc::client::Context::new(&self.commands, function, args).unwrap()
    }
}

mod event_loop;
mod rpc_loop;

pub mod rpc;
