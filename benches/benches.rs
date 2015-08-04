#![feature(test)]

extern crate serde;
extern crate switchboard;
extern crate tempdir;
extern crate test;

#[path="../tests/support/mod.rs"] mod support;

use serde::json;
use support::TestHarness;
use switchboard::client::Client;
use switchboard::ipc;
use switchboard::plugin_buffer;
use test::Bencher;


// On my macbook: 412,692 ns/iter (+/- 33,380)
#[bench]
fn bench_create_and_delete_buffers(b: &mut Bencher) {
    let t = TestHarness::new();
    let client = Client::connect(&t.socket_name);

    b.iter(|| {
        let new_response: plugin_buffer::NewResponse = match client.call(
            "buffer.new", &plugin_buffer::NewRequest).wait().unwrap()
        {
            ipc::RpcResult::Ok(value) => json::from_value(value).unwrap(),
            err => panic!("{:?}", err),
        };

        let _: plugin_buffer::DeleteResponse = match client.call(
            "buffer.delete", &plugin_buffer::DeleteRequest {
            buffer_index: new_response.buffer_index
        }).wait().unwrap() {
            ipc::RpcResult::Ok(value) => json::from_value(value).unwrap(),
            err => panic!("{:?}", err),
        };
    });
}