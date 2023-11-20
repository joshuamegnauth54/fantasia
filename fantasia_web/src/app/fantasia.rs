use std::{io, net::SocketAddr};

use axum::{routing::IntoMakeService, Router, Server};
use hyper::server::conn::AddrIncoming;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::{self, ToSocketAddrs};
use tracing::{debug, info};

use crate::state::State;

pub struct Fantasia {
    router: Router,
    listener: SocketAddr,
    state: State,
}

impl Fantasia {
    pub fn new(listener: SocketAddr, pool: PgPool) -> Fantasia {
        let router = super::router::bind_routes();
        let state = State { pool };

        Fantasia {
            router,
            listener,
            state,
        }
    }

    /// Build a [Fantasia] instance from network addresses.
    ///
    /// # Arguments
    /// * `addr` - Bind the server to this address.
    /// * `options` - Options for the Postgres [sqlx::PgPool]
    /// * `url` - Postgres server URL
    #[tracing::instrument(skip(addr))]
    pub async fn new_from_addr(
        addr: impl ToSocketAddrs,
        options: PgPoolOptions,
        url: &str,
    ) -> io::Result<Fantasia> {
        info!("Retrieving socket addresses");

        #[cfg(debug_assertions)]
        let listener = {
            let addrs: Vec<_> = net::lookup_host(addr).await?.collect();
            for sockaddr in &addrs {
                debug!("Socket address: {sockaddr}");
            }

            *addrs.first().ok_or(io::ErrorKind::AddrNotAvailable)?
        };

        #[cfg(not(debug_assertions))]
        let listener = addr
            .to_socket_addrs()?
            .next()
            .ok_or(io::ErrorKind::AddrNotAvailable)?;
        info!("Using address: {listener}");

        info!("Connecting to Postgres database at `{url}`");
        let pool = options
            .connect(url)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        info!("Successfully connected to the Postgres server");

        Ok(Fantasia::new(listener, pool))
    }

    pub fn into_server(
        self,
    ) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, hyper::Error> {
        Server::try_bind(&self.listener).map(|server| server.serve(self.router.into_make_service()))
    }
}

impl TryInto<Server<AddrIncoming, IntoMakeService<Router>>> for Fantasia {
    type Error = hyper::Error;

    fn try_into(self) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, Self::Error> {
        self.into_server()
    }
}

#[cfg(test)]
mod tests {
    use super::Fantasia;
}
