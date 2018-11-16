/**************************************************************
* CREATE TABLE
**************************************************************/
-- Stores list of immutable subgraphs
CREATE TABLE IF NOT EXISTS subgraphs (
    id VARCHAR PRIMARY KEY,
    network_name VARCHAR NOT NULL,
    latest_block_hash VARCHAR NOT NULL,
    latest_block_number BIGINT NOT NULL
);

-- Maps names to immutable subgraph versions (IDs) and node IDs.
CREATE TABLE IF NOT EXISTS subgraph_deployments (
    deployment_name VARCHAR PRIMARY KEY,
    subgraph_id VARCHAR UNIQUE NOT NULL,
    node_id VARCHAR NOT NULL
);
