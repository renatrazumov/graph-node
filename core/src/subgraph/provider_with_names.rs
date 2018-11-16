use std::collections::HashSet;

use graph::prelude::{
    SubgraphProvider as SubgraphProviderTrait,
    SubgraphProviderWithNames as SubgraphProviderWithNamesTrait, *,
};

pub struct SubgraphProviderWithNames<P, S> {
    logger: slog::Logger,
    provider: Arc<P>,
    store: Arc<S>,
    node_id: NodeId,
}

impl<P, S> Clone for SubgraphProviderWithNames<P, S> {
    fn clone(&self) -> Self {
        Self {
            logger: self.logger.clone(),
            provider: self.provider.clone(),
            store: self.store.clone(),
            node_id: self.node_id.clone(),
        }
    }
}

impl<P, S> SubgraphProviderWithNames<P, S>
where
    P: SubgraphProviderTrait,
    S: SubgraphDeploymentStore,
{
    pub fn init(
        logger: slog::Logger,
        provider: Arc<P>,
        store: Arc<S>,
        node_id: NodeId,
    ) -> impl Future<Item = Self, Error = Error> {
        let logger = logger.new(o!("component" => "SubgraphProviderWithNames"));
        let logger_clone = logger.clone();

        // Create the named subgraph provider
        let provider = SubgraphProviderWithNames {
            logger,
            provider,
            store,
            node_id,
        };

        // Start event stream before starting deployed subgraphs
        let deployment_event_stream = provider.store.deployment_events(provider.node_id.clone());

        let provider_clone = provider.clone();
        tokio::spawn(future::lazy(move || {
            deployment_event_stream
                .from_err()
                .for_each(
                    move |deployment_event| -> Box<Future<Item = (), Error = _> + Send> {
                        match deployment_event {
                            DeploymentEvent::Add {
                                deployment_name: _,
                                subgraph_id,
                                node_id: event_node_id,
                            } => {
                                assert_eq!(event_node_id, provider_clone.node_id);

                                // TODO handle error, retry? or just log?
                                Box::new(provider_clone.provider.start(subgraph_id).then(
                                    |result| match result {
                                        Ok(()) => Ok(()),
                                        Err(SubgraphProviderError::AlreadyRunning(_)) => Ok(()),
                                        Err(e) => Err(e),
                                    },
                                ))
                            }
                            DeploymentEvent::Remove {
                                deployment_name: _,
                                subgraph_id,
                                node_id: event_node_id,
                            } => {
                                assert_eq!(event_node_id, provider_clone.node_id);

                                // TODO handle error, retry? or just log?
                                Box::new(provider_clone.provider.stop(subgraph_id).then(|result| {
                                    match result {
                                        Ok(()) => Ok(()),
                                        Err(SubgraphProviderError::NotRunning(_)) => Ok(()),
                                        Err(e) => Err(e),
                                    }
                                }))
                            }
                        }
                    },
                ).map_err(move |e| {
                    error!(logger_clone, "deployment event stream failed: {}", e);
                    panic!("deployment event stream error");
                })
        }));

        // Deploy named subgraph found in store
        provider.start_deployed_subgraphs().map(|()| provider)
    }

    fn start_deployed_subgraphs(&self) -> impl Future<Item = (), Error = Error> {
        let self_clone = self.clone();

        future::result(self.store.read_by_node_id(self.node_id.clone())).and_then(
            move |names_and_subgraph_ids| {
                let self_clone = self_clone.clone();

                let subgraph_ids = names_and_subgraph_ids
                    .into_iter()
                    .map(|(_name, id)| id)
                    .collect::<HashSet<SubgraphId>>();

                stream::iter_ok(subgraph_ids)
                    .for_each(move |id| self_clone.provider.start(id).from_err())
            },
        )
    }
}

impl<P, S> SubgraphProviderWithNamesTrait for SubgraphProviderWithNames<P, S>
where
    P: SubgraphProviderTrait,
    S: SubgraphDeploymentStore,
{
    fn deploy(
        &self,
        name: SubgraphDeploymentName,
        id: SubgraphId,
    ) -> Box<Future<Item = (), Error = SubgraphProviderError> + Send + 'static> {
        Box::new(
            future::result(
                self.store
                    .write(name.clone(), id.clone(), self.node_id.clone()),
            ).from_err(),
        )
    }

    fn remove(
        &self,
        name: SubgraphDeploymentName,
    ) -> Box<Future<Item = (), Error = SubgraphProviderError> + Send + 'static> {
        Box::new(
            future::result(self.store.remove(name.clone()))
                .from_err()
                .and_then(move |did_remove| {
                    if did_remove {
                        Ok(())
                    } else {
                        Err(SubgraphProviderError::NameNotFound(name.to_string()))
                    }
                }),
        )
    }

    fn list(&self) -> Result<Vec<(SubgraphDeploymentName, SubgraphId)>, Error> {
        self.store.read_by_node_id(self.node_id.clone())
    }
}
