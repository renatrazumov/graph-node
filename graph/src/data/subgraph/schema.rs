//! See `core/src/subgraph/subgraphs.graphql` for corresponding graphql schema.

use super::SubgraphId;
use components::store::{EntityKey, EntityOperation, Store};
use data::store::Value;
use failure::Error;
use std::collections::HashMap;

/// ID of the subgraph of subgraphs.
pub const SUBGRAPHS_ID: &str = "subgraphs";
const EVENT_SOURCE: &str = "subgraph-added";

#[derive(Debug)]
pub struct SubgraphEntity {
    id: SubgraphId,
    manifest: SubgraphManifest,
    created_at: u64,
}

impl SubgraphEntity {
    pub fn new(source_manifest: &super::SubgraphManifest, created_at: u64) -> Self {
        Self {
            id: source_manifest.id.clone(),
            manifest: SubgraphManifest::from(source_manifest),
            created_at,
        }
    }

    pub fn write_to_store(self, store: &impl Store) -> Result<(), Error> {
        let mut entity = HashMap::new();
        entity.insert("id".to_owned(), self.id.clone().into());
        let manifest_id = format!("{}-manifest", self.id);
        entity.insert(
            "manifest".to_owned(),
            self.manifest.write_to_store(store, manifest_id)?.into(),
        );
        entity.insert("createdAt".to_owned(), self.created_at.into());

        store.apply_set_operation(
            EntityOperation::Set {
                key: EntityKey {
                    subgraph_id: SUBGRAPHS_ID.to_owned(),
                    entity_type: "Subgraph".to_owned(),
                    entity_id: self.id,
                },
                data: entity.into(),
            },
            EVENT_SOURCE.to_owned(),
        )?;
        Ok(())
    }
}

#[derive(Debug)]
struct SubgraphManifest {
    spec_version: String,
    description: Option<String>,
    repository: Option<String>,
    schema: String,
    data_sources: Vec<EthereumContractDataSource>,
}

impl SubgraphManifest {
    // Returns the id in the store.
    fn write_to_store(self, store: &impl Store, id: String) -> Result<String, Error> {
        let mut entity = HashMap::new();
        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("specVersion".to_owned(), self.spec_version.into());
        entity.insert("description".to_owned(), self.description.into());
        entity.insert("repository".to_owned(), self.repository.into());
        entity.insert("schema".to_owned(), self.schema.into());

        let mut data_sources: Vec<Value> = Vec::new();
        for (i, data_source) in self.data_sources.into_iter().enumerate() {
            let data_source_id = format!("{}-data-source-{}", id, i);
            data_sources.push(data_source.write_to_store(store, data_source_id)?.into())
        }
        entity.insert("dataSources".to_owned(), data_sources.into());

        store.apply_set_operation(
            EntityOperation::Set {
                key: EntityKey {
                    subgraph_id: SUBGRAPHS_ID.to_owned(),
                    entity_type: "SubgraphManifest".to_owned(),
                    entity_id: id.clone(),
                },
                data: entity.into(),
            },
            EVENT_SOURCE.to_owned(),
        )?;
        Ok(id)
    }
}

impl<'a> From<&'a super::SubgraphManifest> for SubgraphManifest {
    fn from(manifest: &'a super::SubgraphManifest) -> Self {
        Self {
            spec_version: manifest.spec_version.clone(),
            description: manifest.description.clone(),
            repository: manifest.repository.clone(),
            schema: manifest.schema.document.clone().to_string(),
            data_sources: manifest.data_sources.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug)]
struct EthereumContractDataSource {
    kind: String,
    network: Option<String>,
    name: String,
    source: EthereumContractSource,
    mapping: EthereumContractMapping,
}

impl EthereumContractDataSource {
    // Returns the id in the store.
    fn write_to_store(self, store: &impl Store, id: String) -> Result<String, Error> {
        let mut entity = HashMap::new();
        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("kind".to_owned(), self.kind.into());
        entity.insert("network".to_owned(), self.network.into());
        entity.insert("name".to_owned(), self.name.into());
        entity.insert(
            "source".to_owned(),
            self.source
                .write_to_store(store, format!("{}-source", id))?
                .into(),
        );
        entity.insert(
            "mapping".to_owned(),
            self.mapping
                .write_to_store(store, format!("{}-mapping", id))?
                .into(),
        );

        store.apply_set_operation(
            EntityOperation::Set {
                key: EntityKey {
                    subgraph_id: SUBGRAPHS_ID.to_owned(),
                    entity_type: "EthereumContractDataSource".to_owned(),
                    entity_id: id.clone(),
                },
                data: entity.into(),
            },
            EVENT_SOURCE.to_owned(),
        )?;
        Ok(id)
    }
}

impl<'a> From<&'a super::DataSource> for EthereumContractDataSource {
    fn from(data_source: &'a super::DataSource) -> Self {
        Self {
            kind: data_source.kind.clone(),
            name: data_source.name.clone(),
            network: data_source.network.clone(),
            source: data_source.source.clone().into(),
            mapping: EthereumContractMapping::from(&data_source.mapping),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct EthereumContractSource {
    address: super::Address,
    abi: String,
}

impl EthereumContractSource {
    // Returns the id in the store.
    fn write_to_store(self, store: &impl Store, id: String) -> Result<String, Error> {
        let mut entity = HashMap::new();
        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("address".to_owned(), self.address.into());
        entity.insert("abi".to_owned(), self.abi.into());

        store.apply_set_operation(
            EntityOperation::Set {
                key: EntityKey {
                    subgraph_id: SUBGRAPHS_ID.to_owned(),
                    entity_type: "EthereumContractSource".to_owned(),
                    entity_id: id.clone(),
                },
                data: entity.into(),
            },
            EVENT_SOURCE.to_owned(),
        )?;
        Ok(id)
    }
}

impl From<super::Source> for EthereumContractSource {
    fn from(source: super::Source) -> Self {
        Self {
            address: source.address,
            abi: source.abi,
        }
    }
}

#[derive(Debug)]
struct EthereumContractMapping {
    kind: String,
    api_version: String,
    language: String,
    file: String,
    entities: Vec<String>,
    abis: Vec<EthereumContractAbi>,
    event_handlers: Vec<EthereumContractEventHandler>,
}

impl EthereumContractMapping {
    // Returns the id in the store.
    fn write_to_store(self, store: &impl Store, id: String) -> Result<String, Error> {
        let mut entity = HashMap::new();
        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("kind".to_owned(), self.kind.into());
        entity.insert("apiVersion".to_owned(), self.api_version.into());
        entity.insert("language".to_owned(), self.language.into());
        entity.insert("file".to_owned(), self.file.into());

        let mut abis: Vec<Value> = Vec::new();
        for (i, abi) in self.abis.into_iter().enumerate() {
            let abi_id = format!("{}-abi-{}", id, i);
            abis.push(abi.write_to_store(store, abi_id)?.into())
        }
        entity.insert("abis".to_owned(), abis.into());

        let entities: Vec<Value> = self.entities.into_iter().map(Value::from).collect();
        entity.insert("entities".to_owned(), entities.into());

        let mut event_handlers: Vec<Value> = Vec::new();
        for (i, event_handler) in self.event_handlers.into_iter().enumerate() {
            let handler_id = format!("{}-event-handler-{}", id, i);
            event_handlers.push(event_handler.write_to_store(store, handler_id)?.into())
        }
        entity.insert("eventHandlers".to_owned(), event_handlers.into());

        store.apply_set_operation(
            EntityOperation::Set {
                key: EntityKey {
                    subgraph_id: SUBGRAPHS_ID.to_owned(),
                    entity_type: "EthereumContractMapping".to_owned(),
                    entity_id: id.clone(),
                },
                data: entity.into(),
            },
            EVENT_SOURCE.to_owned(),
        )?;
        Ok(id)
    }
}

impl<'a> From<&'a super::Mapping> for EthereumContractMapping {
    fn from(mapping: &'a super::Mapping) -> Self {
        Self {
            kind: mapping.kind.clone(),
            api_version: mapping.api_version.clone(),
            language: mapping.language.clone(),
            file: mapping.link.link.clone(),
            entities: mapping.entities.clone(),
            abis: mapping.abis.iter().map(Into::into).collect(),
            event_handlers: mapping
                .event_handlers
                .clone()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug)]
struct EthereumContractAbi {
    name: String,
    file: String,
}

impl EthereumContractAbi {
    // Returns the id in the store.
    fn write_to_store(self, store: &impl Store, id: String) -> Result<String, Error> {
        let mut entity = HashMap::new();
        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("name".to_owned(), self.name.into());
        entity.insert("file".to_owned(), self.file.into());

        store.apply_set_operation(
            EntityOperation::Set {
                key: EntityKey {
                    subgraph_id: SUBGRAPHS_ID.to_owned(),
                    entity_type: "EthereumContractAbi".to_owned(),
                    entity_id: id.clone(),
                },
                data: entity.into(),
            },
            EVENT_SOURCE.to_owned(),
        )?;
        Ok(id)
    }
}

impl<'a> From<&'a super::MappingABI> for EthereumContractAbi {
    fn from(abi: &'a super::MappingABI) -> Self {
        Self {
            name: abi.name.clone(),
            file: abi.link.link.clone(),
        }
    }
}

#[derive(Debug)]
struct EthereumContractEventHandler {
    event: String,
    handler: String,
}

impl EthereumContractEventHandler {
    // Returns the id in the store.
    fn write_to_store(self, store: &impl Store, id: String) -> Result<String, Error> {
        let mut entity = HashMap::new();
        entity.insert("id".to_owned(), id.clone().into());
        entity.insert("event".to_owned(), self.event.into());
        entity.insert("handler".to_owned(), self.handler.into());

        store.apply_set_operation(
            EntityOperation::Set {
                key: EntityKey {
                    subgraph_id: SUBGRAPHS_ID.to_owned(),
                    entity_type: "EthereumContractEventHandler".to_owned(),
                    entity_id: id.clone(),
                },
                data: entity.into(),
            },
            EVENT_SOURCE.to_owned(),
        )?;
        Ok(id)
    }
}

impl From<super::MappingEventHandler> for EthereumContractEventHandler {
    fn from(event_handler: super::MappingEventHandler) -> Self {
        Self {
            event: event_handler.event,
            handler: event_handler.handler,
        }
    }
}
