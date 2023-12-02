use std::{future::Future, io, iter, net::SocketAddr};

use axum::{
    serve::{self},
    Router,
};
use futures::future::{join_all, JoinAll};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::{self, TcpListener, ToSocketAddrs};
use tracing::{debug, info};

use crate::{state::State, Serve};

pub struct Fantasia {
    router: Router,
    sockets: Vec<SocketAddr>,
}

impl Fantasia {
    /// Construct [Fantasia] instances from parsed [SocketAddr]s.
    #[tracing::instrument]
    pub fn new(sockets: &[SocketAddr], pool: PgPool) -> Fantasia {
        let sockets = sockets.to_vec();
        debug!("{} socket addresses", sockets.len());

        let state = State { pool };
        let router = super::router::bind_routes(state);

        Fantasia { router, sockets }
    }

    /// Build [Fantasia] instances by resolving network addresses and connecting to Postgres.
    ///
    /// The resulting instances must be spawned in order to start the web app.
    ///
    /// # Arguments
    /// * `addrs` - Bind the server to these addresses.
    /// * `options` - Options for the Postgres [sqlx::PgPool]
    /// * `url` - Postgres server URL
    #[tracing::instrument(skip(addrs))]
    pub async fn new_from_addr(
        addrs: impl ToSocketAddrs,
        options: PgPoolOptions,
        url: &SecretString,
    ) -> io::Result<Fantasia> {
        info!("Retrieving socket addresses");

        // Asynchronously look up provided addresses.
        // I'm not using the standard library's [std::net::ToSocketAddrs] because that blocks the
        // executor.
        let addrs: Vec<_> = net::lookup_host(addrs).await?.collect();
        if addrs.is_empty() {
            Err(io::ErrorKind::AddrNotAvailable)?;
        }
        for sockaddr in &addrs {
            info!("Using address: {sockaddr}");
        }

        info!("Connecting to Postgres database at `{url:?}`");
        let pool = options
            .connect(url.expose_secret())
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        info!("Successfully connected to the Postgres server");

        Ok(Fantasia::new(&addrs, pool))
    }

    pub fn into_server(self) -> JoinAll<impl Future<Output = io::Result<Serve>>> {
        let Self { sockets, router } = self;

        join_all(
            sockets
                .into_iter()
                .map(TcpListener::bind)
                .zip(iter::repeat(router))
                .map(|(listener, router)| async {
                    listener.await.map(|listener| {
                        serve::serve(
                            listener,
                            router.into_make_service_with_connect_info::<SocketAddr>(),
                        )
                    })
                }),
        )
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
