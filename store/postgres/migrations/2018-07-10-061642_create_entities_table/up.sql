/**************************************************************
* CREATE TABLE
**************************************************************/
CREATE TABLE IF NOT EXISTS entities (
     id VARCHAR NOT NULL,
     data_source VARCHAR NOT NULL,
     entity VARCHAR NOT NULL,
     block_hash VARCHAR(40) DEFAULT NULL,
     data jsonb NOT NULL,
     PRIMARY KEY (id, data_source, entity)
 );