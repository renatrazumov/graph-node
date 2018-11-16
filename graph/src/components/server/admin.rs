use std::io;
use std::sync::Arc;

use prelude::Logger;

/// Common trait for JSON-RPC admin server implementations.
pub trait JsonRpcServer<P> {
    type Server;

    fn serve(port: u16, provider: Arc<P>, logger: Logger) -> Result<Self::Server, io::Error>;
}
