#![feature(assert_matches)]

use std::fmt::Debug;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::anyhow;
use constants::{MAX_PORT, MIN_PORT};
use derive_more::Display;
use lazy_static::lazy_static;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Response};
use serde_json::json;
use starknet_accounts::{Execution, SingleOwnerAccount};
use starknet_core::types::InvokeTransactionResult;
use starknet_providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_providers::Provider;
use starknet_signers::LocalWallet;
use thiserror::Error;
use tokio::sync::Mutex;
use url::Url;

/// Constants (addresses, contracts...)
pub mod constants;
/// Starknet related utilities
pub mod utils;

type TransactionExecution<'a> = Execution<'a, SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>>;

lazy_static! {
        /// This is to prevent TOCTOU errors; i.e. one background madara node might find one
        /// port to be free, and while it's trying to start listening to it, another instance
        /// finds that it's free and tries occupying it
        /// Using the mutex in `get_free_port_listener` might be safer than using no mutex at all,
        /// but not sufficiently safe
        static ref BACKGROUND_MADARA_MUTEX: Mutex<()> = Mutex::new(());
}

#[derive(Debug)]
/// A wrapper over the Madara process handle, reqwest client and request counter
///
/// When this struct goes out of scope, it's `Drop` impl
/// will take care of killing the Madara process.
pub struct MadaraClient {
    process: Child,
    client: Client,
    rpc_request_count: AtomicUsize,
    starknet_client: JsonRpcClient<HttpTransport>,
    port: u16,
}

#[derive(Display)]
pub enum ExecutionStrategy {
    Native,
    Wasm,
}

#[derive(Error, Debug)]
pub enum TestError {
    #[error("No free ports")]
    NoFreePorts,
}

impl Drop for MadaraClient {
    fn drop(&mut self) {
        self.process.kill().expect("Cannot kill process");
    }
}

fn get_free_port() -> Result<u16, TestError> {
    for port in MIN_PORT..=MAX_PORT {
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
            return Ok(listener.local_addr().expect("No local addr").port());
        }
        // otherwise port is occupied
    }

    Err(TestError::NoFreePorts)
}

impl MadaraClient {
    async fn init(execution: ExecutionStrategy) -> Result<Self, TestError> {
        // we keep the reference, otherwise the mutex unlocks immediately
        let _mutex_guard = BACKGROUND_MADARA_MUTEX.lock().await;

        let madara_path = Path::new("../target/release/madara");
        assert!(
            madara_path.exists(),
            "could not find the madara binary at `{}`",
            madara_path.to_str().expect("madara_path must be a valid path")
        );

        let free_port = get_free_port()?;

        let child_handle = Command::new(madara_path.to_str().unwrap())
                // Silence Madara stdout and stderr
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .args([
                    "--alice",
                    "--sealing=manual",
                    &format!("--execution={execution}"),
                    "--chain=dev",
                    "--tmp",
                    &format!("--rpc-port={free_port}"),
                ])
                .spawn()
                .unwrap();

        let host = &format!("http://localhost:{free_port}");

        let starknet_client = JsonRpcClient::new(HttpTransport::new(Url::parse(host).expect("Invalid JSONRPC Url")));

        Ok(MadaraClient {
            process: child_handle,
            client: Client::new(),
            starknet_client,
            rpc_request_count: Default::default(),
            port: free_port,
        })
    }

    pub async fn new(execution: ExecutionStrategy) -> Self {
        let madara = Self::init(execution).await.expect("Couldn't start Madara Node");

        // Wait until node is ready
        loop {
            match madara.health().await {
                Ok(is_ready) if is_ready => break,
                _ => {}
            }
        }

        madara
    }

    pub async fn run_to_block(&self, target_block: u64) -> anyhow::Result<()> {
        let mut current_block = self.starknet_client.block_number().await?;

        if current_block >= target_block {
            return Err(anyhow!("target_block must be in the future"));
        }

        while current_block < target_block {
            self.create_empty_block().await?;
            current_block += 1;
        }

        Ok(())
    }

    pub async fn create_n_blocks(&self, mut n: u64) -> anyhow::Result<()> {
        while n > 0 {
            self.create_empty_block().await?;
            n -= 1;
        }

        Ok(())
    }

    async fn call_rpc(&self, mut body: serde_json::Value) -> reqwest::Result<Response> {
        let body = body.as_object_mut().expect("the body should be an object");
        let current_id = self.rpc_request_count.fetch_add(1, Ordering::Relaxed);
        body.insert("id".to_string(), current_id.into());
        body.insert("jsonrpc".to_string(), "2.0".into());

        let body = serde_json::to_string(&body).expect("the json body must be serializable");

        let response = self
            .client
            .post(&format!("http://localhost:{0}", self.port))
            .header(CONTENT_TYPE, "application/json; charset=utf-8")
            .body(body)
            .send()
            .await?;

        Ok(response)
    }

    pub fn get_starknet_client(&self) -> &JsonRpcClient<HttpTransport> {
        &self.starknet_client
    }
}

// Substrate RPC
impl MadaraClient {
    pub async fn create_empty_block(&self) -> anyhow::Result<()> {
        let body = json!({
            "method": "engine_createBlock",
            "params": [true, true],
        });

        let response = self.call_rpc(body).await?;
        // TODO: read actual error from response
        response.status().is_success().then_some(()).ok_or(anyhow!("failed to create a new block"))
    }

    pub async fn create_block_with_txs(&self, transactions: Vec<TransactionExecution<'_>>) -> anyhow::Result<()> {
        let body = json!({
            "method": "engine_createBlock",
            "params": [false, true],
        });

        let mut results: Vec<InvokeTransactionResult> = Vec::new();
        for tx in transactions {
            let result = tx.send().await?;
            results.push(result);
        }

        let response = self.call_rpc(body).await?;
        // TODO: read actual error from response
        response.status().is_success().then_some(()).ok_or(anyhow!("failed to create a new block"))
    }

    pub async fn create_block_with_parent(&self, parent_hash: &str) -> anyhow::Result<()> {
        let body = json!({
            "method": "engine_createBlock",
            "params": [json!(true), json!(true), json!(parent_hash)],
        });

        let response = self.call_rpc(body).await?;
        // TODO: read actual error from response
        response.status().is_success().then_some(()).ok_or(anyhow!("failed to create a new block"))
    }

    pub async fn health(&self) -> anyhow::Result<bool> {
        let body = json!({
            "method": "system_health"
        });

        let response = self.call_rpc(body).await?;

        Ok(response.status().is_success())
    }
}
