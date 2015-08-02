use serde::json;
use std::path;
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use super::ipc::{self};
use super::client::{self, Client, RemoteProcedure};
use super::ipc::RpcResultKind;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NewRequest;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NewResponse {
    buffer_index: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NewBuffer {
    buffer_index: usize,
}

struct New {
    client_handle: client::ClientHandle,
    buffers: Arc<RwLock<BuffersManager>>,
}

impl<'a> RemoteProcedure for New {
    fn call(&mut self, args: json::Value) -> RpcResultKind {
        // NOCOM(#sirver): need some: on bad request results
        // NOCOM(#sirver): needs some understanding what happens on extra values.
        let request: NewRequest = json::from_value(args).unwrap();
        let mut buffers = self.buffers.write().unwrap();

        let response = NewResponse {
            buffer_index: buffers.new_buffer(),
        };
        // NOCOM(#sirver): return the response.
        RpcResultKind::Ok
    }
}

struct BuffersManager {
    client_handle: client::ClientHandle,
    next_buffer_index: usize,
    buffers: HashMap<usize, String>,
}

impl BuffersManager {
    fn new(client_handle: client::ClientHandle) -> Self {
        BuffersManager {
            client_handle: client_handle,
            next_buffer_index: 0,
            buffers: HashMap::new(),
        }
    }

    fn new_buffer(&mut self) -> usize {
        let current_buffer_index = self.next_buffer_index;
        self.next_buffer_index += 1;

        self.buffers.insert(current_buffer_index, String::new());
        // NOCOM(#sirver): having a broadcast function would be nice.
        self.client_handle.call("core.broadcast", &json::to_value(&NewBuffer {
            buffer_index: current_buffer_index,
        })).wait();
        current_buffer_index
    }
}

pub struct BufferPlugin<'a> {
    client: Client<'a>,
    buffers: Arc<RwLock<BuffersManager>>,
}

impl<'a> BufferPlugin<'a> {
    // NOCOM(#sirver): is 'b needed?
    pub fn new(socket_name: &path::Path) -> Self {
        let client = Client::connect(socket_name);

        let mut plugin = BufferPlugin {
            buffers: Arc::new(RwLock::new(BuffersManager::new(client.client_handle()))),
            client: client,
        };

        let new = Box::new(New {
            client_handle: plugin.client.client_handle(),
            buffers: plugin.buffers.clone(),
        });
        plugin.client.register_function("buffer.new", new);
        plugin
    }
}
