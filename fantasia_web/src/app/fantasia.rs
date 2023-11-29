use std::{io, net::SocketAddr};

use axum::{
    body::Body,
    extract::{
        connect_info::{Connected, IntoMakeServiceWithConnectInfo},
        ConnectInfo,
    },
    http::Request,
    middleware::AddExtension,
    routing::IntoMakeService,
    serve::{self, IncomingStream, Serve},
    Router, ServiceExt,
};
use hyper::service::Service;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::{self, TcpListener, ToSocketAddrs};
use tracing::{debug, info};

use crate::state::State;

pub struct Fantasia {
    router: Router,
    sockets: Vec<SocketAddr>,
}

impl Fantasia {
    pub fn new(sockets: &[SocketAddr], pool: PgPool) -> Fantasia {
        let sockets = sockets.into_iter().map(|socket| *socket).collect();
        let state = State { pool };
        let router = super::router::bind_routes(state);

        Fantasia { router, sockets }
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
        url: &SecretString,
    ) -> io::Result<Fantasia> {
        info!("Retrieving socket addresses");

        #[cfg(debug_assertions)]
        let listeners = {
            let addrs: Vec<_> = net::lookup_host(addr).await?.collect();
            for sockaddr in &addrs {
                debug!("Socket address: {sockaddr}");
            }

            // *addrs.first().ok_or(io::ErrorKind::AddrNotAvailable)?
            addrs
        };

        #[cfg(not(debug_assertions))]
        let listeners = addr.to_socket_addrs()?.collect();
        // info!("Using address: {listener}");

        info!("Connecting to Postgres database at `{url:?}`");
        let pool = options
            .connect(url.expose_secret())
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        info!("Successfully connected to the Postgres server");

        Ok(Fantasia::new(&listeners, pool))
    }

    pub async fn into_server(
        self,
    ) -> Serve<
        IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
        AddExtension<Router, ConnectInfo<SocketAddr>>,
    > {
        let socket = self.sockets.into_iter().next().unwrap();
        let socket = TcpListener::bind(socket).await.unwrap();

        serve::serve(socket, self.router.into_make_service_with_connect_info())
    }
}

// impl TryInto<Server<AddrIncoming, IntoMakeService<Router>>> for Fantasia {
//     type Error = hyper::Error;
//
//     fn try_into(self) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, Self::Error> {
//         self.into_server()
//     }
// }

#[cfg(test)]
mod tests {
    use super::Fantasia;
}
