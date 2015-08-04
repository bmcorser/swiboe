use serde::json;
use support::TestHarness;
use switchboard::client::{RemoteProcedure, Client};
use switchboard::ipc::RpcResult;
use switchboard::plugin_buffer;

fn create_buffer(client: &Client, expected_index: usize) {
    let request = plugin_buffer::NewRequest;
    let rpc = client.call("buffer.new", &request);
    assert_eq!(rpc.wait().unwrap(), RpcResult::success(plugin_buffer::NewResponse {
        buffer_index: expected_index,
    }));

    // NOCOM(#sirver): check for callback call.
}

#[test]
fn buffer_new() {
    let t = TestHarness::new();
    let client = Client::connect(&t.socket_name);
    create_buffer(&client, 0);
}

#[test]
fn buffer_delete() {
    let t = TestHarness::new();
    let client = Client::connect(&t.socket_name);
    create_buffer(&client, 0);

    let request = plugin_buffer::DeleteRequest {
        buffer_index: 0,
    };
    let rpc = client.call("buffer.delete", &request);
    assert_eq!(rpc.wait().unwrap(), RpcResult::success(()));

    // NOCOM(#sirver): check for callback call.

    // NOCOM(#sirver): add a test for non existing buffer

}