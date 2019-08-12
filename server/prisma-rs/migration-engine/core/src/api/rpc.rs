use super::{GenericApi, MigrationApi};
use crate::commands::*;
use futures::{
    future::{err, lazy, ok, poll_fn},
    Future,
};
use jsonrpc_core;
use jsonrpc_core::types::error::Error as JsonRpcError;
use jsonrpc_core::IoHandler;
use jsonrpc_core::*;
use jsonrpc_stdio_server::ServerBuilder;
use sql_migration_connector::SqlMigrationConnector;
use std::{io, sync::Arc};
use tokio_threadpool::blocking;

pub struct RpcApi {
    io_handler: jsonrpc_core::IoHandler<()>,
    executor: Arc<dyn GenericApi>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RpcCommand {
    InferMigrationSteps,
    ListMigrations,
    MigrationProgress,
    ApplyMigration,
    UnapplyMigration,
    Reset,
    CalculateDatamodel,
    CalculateDatabaseSteps,
}

impl RpcCommand {
    fn name(&self) -> &'static str {
        match self {
            RpcCommand::InferMigrationSteps => "inferMigrationSteps",
            RpcCommand::ListMigrations => "listMigrations",
            RpcCommand::MigrationProgress => "migrationProgress",
            RpcCommand::ApplyMigration => "applyMigration",
            RpcCommand::UnapplyMigration => "unapplyMigration",
            RpcCommand::Reset => "reset",
            RpcCommand::CalculateDatamodel => "calculateDatamodel",
            RpcCommand::CalculateDatabaseSteps => "calculateDatabaseSteps",
        }
    }
}

impl RpcApi {
    pub fn new(datamodel: &str) -> crate::Result<RpcApi> {
        let config = datamodel::load_configuration(datamodel)?;

        let source = config.datasources.first().ok_or(CommandError::DataModelErrors {
            code: 1000,
            errors: vec!["There is no datasource in the configuration.".to_string()],
        })?;

        let connector = match source.connector_type().as_ref() {
            "sqlite" => SqlMigrationConnector::sqlite(&source.url())?,
            "postgresql" => SqlMigrationConnector::postgres(&source.url())?,
            "mysql" => SqlMigrationConnector::mysql(&source.url())?,
            x => unimplemented!("Connector {} is not supported yet", x),
        };

        let mut rpc_api = RpcApi {
            io_handler: IoHandler::new(),
            executor: Arc::new(MigrationApi::new(connector)?),
        };

        rpc_api.add_command_handler(RpcCommand::ApplyMigration);
        rpc_api.add_command_handler(RpcCommand::InferMigrationSteps);
        rpc_api.add_command_handler(RpcCommand::ListMigrations);
        rpc_api.add_command_handler(RpcCommand::MigrationProgress);
        rpc_api.add_command_handler(RpcCommand::MigrationProgress);
        rpc_api.add_command_handler(RpcCommand::UnapplyMigration);
        rpc_api.add_command_handler(RpcCommand::Reset);
        rpc_api.add_command_handler(RpcCommand::CalculateDatamodel);
        rpc_api.add_command_handler(RpcCommand::CalculateDatabaseSteps);

        Ok(rpc_api)
    }

    /// Block the thread and handle IO in async until EOF.
    pub fn start_server(self) {
        ServerBuilder::new(self.io_handler).build()
    }

    /// Handle one request
    pub fn handle(&self) -> crate::Result<String> {
        let mut json_is_complete = false;
        let mut input = String::new();

        while !json_is_complete {
            io::stdin().read_line(&mut input)?;
            json_is_complete = serde_json::from_str::<serde_json::Value>(&input).is_ok();
        }

        let result = self
            .io_handler
            .handle_request_sync(&input)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Reading from stdin failed."))?;

        Ok(result)
    }

    fn add_command_handler(&mut self, cmd: RpcCommand) {
        let executor = Arc::clone(&self.executor);

        self.io_handler.add_method(cmd.name(), move |params: Params| {
            Self::create_handler(&executor, cmd, params)
        });
    }

    fn create_handler(
        executor: &Arc<dyn GenericApi>,
        cmd: RpcCommand,
        params: Params,
    ) -> impl Future<Item = serde_json::Value, Error = JsonRpcError> {
        let executor = Arc::clone(executor);

        lazy(move || {
            poll_fn(move || {
                blocking(|| {
                    let response_json = match cmd {
                        RpcCommand::InferMigrationSteps => {
                            let input: InferMigrationStepsInput = params.clone().parse()?;
                            let result = executor.infer_migration_steps(&input)?;

                            serde_json::to_value(result).expect("Rendering of RPC response failed")
                        }
                        RpcCommand::ListMigrations => {
                            let result = executor.list_migrations(&serde_json::Value::Null)?;

                            serde_json::to_value(result).expect("Rendering of RPC response failed")
                        }
                        RpcCommand::MigrationProgress => {
                            let input: MigrationProgressInput = params.clone().parse()?;
                            let result = executor.migration_progress(&input)?;

                            serde_json::to_value(result).expect("Rendering of RPC response failed")
                        }
                        RpcCommand::ApplyMigration => {
                            let input: ApplyMigrationInput = params.clone().parse()?;
                            let result = executor.apply_migration(&input)?;

                            serde_json::to_value(result).expect("Rendering of RPC response failed")
                        }
                        RpcCommand::UnapplyMigration => {
                            let input: UnapplyMigrationInput = params.clone().parse()?;
                            let result = executor.unapply_migration(&input)?;

                            serde_json::to_value(result).expect("Rendering of RPC response failed")
                        }
                        RpcCommand::Reset => {
                            let result = executor.reset(&serde_json::Value::Null)?;

                            serde_json::to_value(result).expect("Rendering of RPC response failed")
                        }
                        RpcCommand::CalculateDatamodel => {
                            let input: CalculateDatamodelInput = params.clone().parse()?;
                            let result = executor.calculate_datamodel(&input)?;

                            serde_json::to_value(result).expect("Rendering of RPC response failed")
                        }
                        RpcCommand::CalculateDatabaseSteps => {
                            let input: CalculateDatabaseStepsInput = params.clone().parse()?;
                            let result = executor.calculate_database_steps(&input)?;

                            serde_json::to_value(result).expect("Rendering of RPC response failed")
                        }
                    };

                    Ok(response_json)
                })
            })
        })
        .then(|res| match res {
            // dumdidum futures 0.1 we love <3
            Ok(Ok(val)) => ok(val),
            Ok(Err(val)) => err(val),
            Err(val) => {
                println!("Fishy stuff...");
                panic!(val)
            }, // When the threadpool is full of work, why not just die...
        })
    }
}

impl From<crate::error::Error> for JsonRpcError {
    fn from(error: crate::error::Error) -> Self {
        match error {
            crate::error::Error::CommandError(command_error) => {
                let json = serde_json::to_value(command_error).unwrap();

                JsonRpcError {
                    code: jsonrpc_core::types::error::ErrorCode::ServerError(4466),
                    message: "An error happened. Check the data field for details.".to_string(),
                    data: Some(json),
                }
            }
            err => panic!(
                "An unexpected error happened. Maybe we should build a handler for these kind of errors? {:?}",
                err
            ),
        }
    }
}
