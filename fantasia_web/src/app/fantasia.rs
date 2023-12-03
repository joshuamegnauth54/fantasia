use std::{future::Future, io, iter, net::SocketAddr};

use axum::{
    serve::{self},
    Router,
};
use futures::future::{join_all, JoinAll};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::{self, TcpListener, ToSocketAddrs};
use tracing::{debug, info, trace};

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

    /// Build a running server from a [Fantasia] instance.
    #[tracing::instrument(skip(self))]
    pub fn into_server(self) -> JoinAll<impl Future<Output = io::Result<Serve>>> {
        trace!("Binding to sockets");

        let Self { sockets, router } = self;

        join_all(
            sockets
                .into_iter()
                // `router` needs to be cloned and moved into the async closure
                .zip(iter::repeat(router))
                .inspect(|(addr, _)| info!("Asynchronously binding to socket address: {addr}"))
                // I'm not sure how to return a Result<JoinAll<_>, _> that simply evaluates to a
                // future that yields `Serve`. This returns
                // `JoinAll<impl Future<Output = io::Result<Serve>>>`
                // which is not ideal because the future that binds the sockets must be evaluated
                // followed by handling any errors followed by awaiting the actual servers
                // (Actually, this may be a good thing for maximum flexibility but it seems kind of
                // ugly to me...but what do I know?)
                .map(|(addr, router)| async move {
                    TcpListener::bind(addr).await.map(|listener| {
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
