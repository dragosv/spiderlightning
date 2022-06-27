pub mod resource;
use std::collections::HashMap;

use anyhow::Result;
use resource::{DataT, ResourceConfig, ResourceMap, RuntimeResource};
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::{StringArrayError, WasiCtx};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::*;

/// A wasmtime runtime context to be passed to a wasm module.
#[derive(Default)]
pub struct Context<T> {
    pub wasi: Option<WasiCtx>,
    pub data: HashMap<String, T>,
}

/// A wasmtime-based runtime builder.
pub struct Builder {
    linker: Linker<Context<DataT>>,
    store: Store<Context<DataT>>,
    engine: Engine,
    pub config: Option<Vec<(String, String)>>,
}

impl Builder {
    /// Create a new runtime builder.
    pub fn new_default() -> Result<Self> {
        let wasi = default_wasi()?;
        let engine = Engine::new(&default_config()?)?;
        let mut linker = Linker::new(&engine);
        linker.allow_shadowing(true);
        let ctx = Context {
            wasi: Some(wasi),
            data: HashMap::new(),
        };

        let store = Store::new(&engine, ctx);
        Ok(Self {
            linker,
            store,
            engine,
            config: None,
        })
    }

    /// Link wasi to the wasmtime::Linker
    pub fn link_wasi(&mut self) -> Result<&mut Self> {
        wasmtime_wasi::add_to_linker(&mut self.linker, |cx: &mut Context<_>| {
            cx.wasi.as_mut().unwrap()
        })?;
        Ok(self)
    }

    /// Link a host capability to the wasmtime::Linker
    pub fn link_capability<T: RuntimeResource>(
        &mut self,
        config: ResourceConfig,
    ) -> Result<&mut Self> {
        self.store.data_mut().data.insert(config, T::build_data()?);
        T::add_to_linker(&mut self.linker)?;
        Ok(self)
    }

    pub fn link_resource_map(&mut self, rd_map: ResourceMap) -> Result<&mut Self> {
        for (_k, v) in self.store.data_mut().data.iter_mut() {
            v.add_resource_map(rd_map.clone())?;
        }
        Ok(self)
    }

    /// Instantiate the guest module.
    pub fn build(mut self, module: &str) -> Result<(Store<Context<DataT>>, Instance)> {
        let module = Module::from_file(&self.engine, module)?;
        let instance = self.linker.instantiate(&mut self.store, &module)?;
        Ok((self.store, instance))
    }
}

// TODO (Joe): expose the wasmtime config as a capability?
pub fn default_config() -> Result<Config> {
    let mut config = Config::new();
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.wasm_multi_memory(true);
    config.wasm_module_linking(true);
    Ok(config)
}

// TODO (Joe): expose the wasmtime wasi context as a capability?
pub fn default_wasi() -> Result<WasiCtx, StringArrayError> {
    let mut ctx: WasiCtxBuilder = WasiCtxBuilder::new().inherit_stdio().inherit_args()?;
    ctx = ctx
        .preopened_dir(
            Dir::open_ambient_dir("./target", ambient_authority()).unwrap(),
            "cache",
        )
        .unwrap();

    Ok(ctx.build())
}