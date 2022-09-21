use crate::catalog::DatabaseCatalog;
use crate::errors::Result;
use crate::session::Session;
use datafusion::execution::runtime_env::RuntimeEnv;
use std::sync::Arc;

pub struct Engine {
    catalog: Arc<DatabaseCatalog>,
    runtime: Arc<RuntimeEnv>, // TODO: Per session runtime.
}

impl Engine {
    pub fn new(db_name: impl Into<String>) -> Result<Engine> {
        let runtime = RuntimeEnv::default();

        let catalog = DatabaseCatalog::new(db_name);
        catalog.insert_default_schema()?;

        let catalog = Arc::new(catalog);
        DatabaseCatalog::insert_information_schema(catalog.clone())?;

        Ok(Engine {
            catalog,
            runtime: Arc::new(runtime),
        })
    }

    pub fn new_session(&self) -> Result<Session> {
        Ok(Session::new(self.catalog.clone(), self.runtime.clone()))
    }
}