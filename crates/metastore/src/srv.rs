use crate::database::DatabaseCatalog;
use crate::errors::MetastoreError;
use crate::proto::service::metastore_service_server::MetastoreService;
use crate::proto::service::{
    self, FetchCatalogRequest, FetchCatalogResponse, InitializeCatalogRequest,
    InitializeCatalogResponse, MutateRequest, MutateResponse,
};
use crate::types::service::Mutation;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// Metastore GRPC service.
pub struct Service {
    catalogs: RwLock<HashMap<Uuid, DatabaseCatalog>>, // Fancy!
}

impl Service {
    pub fn new() -> Service {
        Service {
            catalogs: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetastoreService for Service {
    async fn initialize_catalog(
        &self,
        request: Request<InitializeCatalogRequest>,
    ) -> Result<Response<InitializeCatalogResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::from_slice(&req.db_id)
            .map_err(|_| MetastoreError::InvalidDatabaseId(req.db_id))?;

        let catalogs = self.catalogs.read().await;
        if catalogs.contains_key(&id) {
            return Ok(Response::new(InitializeCatalogResponse {
                status: service::initialize_catalog_response::Status::AlreadyLoaded as i32,
            }));
        }
        std::mem::drop(catalogs);

        let catalog = DatabaseCatalog::open(id).await?;
        let mut catalogs = self.catalogs.write().await;

        // We raced, catalog inserted between locks.
        if catalogs.contains_key(&id) {
            return Ok(Response::new(InitializeCatalogResponse {
                status: service::initialize_catalog_response::Status::AlreadyLoaded as i32,
            }));
        }

        catalogs.insert(id, catalog);

        Ok(Response::new(InitializeCatalogResponse {
            status: service::initialize_catalog_response::Status::Initialized as i32,
        }))
    }

    async fn fetch_catalog(
        &self,
        request: Request<FetchCatalogRequest>,
    ) -> Result<Response<FetchCatalogResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::from_slice(&req.db_id)
            .map_err(|_| MetastoreError::InvalidDatabaseId(req.db_id))?;
        let catalogs = self.catalogs.read().await;

        let catalog = catalogs
            .get(&id)
            .ok_or(MetastoreError::MissingCatalog(id))?;

        let state = catalog.get_state().await?;

        Ok(Response::new(FetchCatalogResponse {
            catalog: Some(state.try_into().map_err(MetastoreError::from)?),
        }))
    }

    async fn mutate_catalog(
        &self,
        request: Request<MutateRequest>,
    ) -> Result<Response<MutateResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::from_slice(&req.db_id)
            .map_err(|_| MetastoreError::InvalidDatabaseId(req.db_id))?;

        let catalogs = self.catalogs.read().await;
        let catalog = catalogs
            .get(&id)
            .ok_or(MetastoreError::MissingCatalog(id))?;

        let mutations = req
            .mutations
            .into_iter()
            .map(|m| Mutation::try_from(m).map_err(MetastoreError::from))
            .collect::<Result<_, _>>()?;

        // TODO: Catch error and return status.

        let updated = catalog.try_mutate(req.catalog_version, mutations).await?;

        Ok(Response::new(MutateResponse {
            status: service::mutate_response::Status::Applied as i32,
            catalog: Some(updated.try_into().map_err(MetastoreError::from)?),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::catalog::CatalogState;
    use crate::types::service::{CreateSchema, Mutation};

    fn new_service() -> Service {
        Service::new()
    }

    #[tokio::test]
    async fn fetch_before_init() {
        let svc = new_service();
        svc.fetch_catalog(Request::new(FetchCatalogRequest {
            db_id: Uuid::new_v4().into_bytes().to_vec(),
        }))
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn init_idempotent() {
        let svc = new_service();
        let id_bs = Uuid::new_v4().into_bytes().to_vec();

        svc.initialize_catalog(Request::new(InitializeCatalogRequest {
            db_id: id_bs.clone(),
        }))
        .await
        .unwrap();

        svc.initialize_catalog(Request::new(InitializeCatalogRequest { db_id: id_bs }))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn simple_mutate() {
        let svc = new_service();
        let id = Uuid::new_v4();
        let id_bs = id.into_bytes().to_vec();

        // Initialize.
        svc.initialize_catalog(Request::new(InitializeCatalogRequest {
            db_id: id_bs.clone(),
        }))
        .await
        .unwrap();

        // Fetch initial catalog.
        let resp = svc
            .fetch_catalog(Request::new(FetchCatalogRequest {
                db_id: id_bs.clone(),
            }))
            .await
            .unwrap();
        let resp = resp.into_inner();

        // Mutate (create schema)
        svc.mutate_catalog(Request::new(MutateRequest {
            db_id: id_bs.clone(),
            catalog_version: resp.catalog.unwrap().version,
            mutations: vec![Mutation::CreateSchema(CreateSchema {
                name: "test_schema".to_string(),
            })
            .into()],
        }))
        .await
        .unwrap();

        // Fetch new catalog.
        let resp = svc
            .fetch_catalog(Request::new(FetchCatalogRequest {
                db_id: id_bs.clone(),
            }))
            .await
            .unwrap();
        let resp = resp.into_inner();

        // Check that we got the new schema.
        let state: CatalogState = resp.catalog.unwrap().try_into().unwrap();
        let ent = state
            .entries
            .into_values()
            .into_iter()
            .find(|ent| ent.get_meta().name == "test_schema")
            .unwrap();
        assert!(ent.is_schema())
    }
}
