# bb8-oracle

[`bb8`](https://github.com/djc/bb8) connection pool support for [`oracle`](https://github.com/kubo/rust-oracle). The code has been derived, to a blatant degree, from [`r2d2-oracle`](https://github.com/rursprung/r2d2-oracle).

Since `oracle` operates synchronously, all pool operations are moved to blocking threads using [`tokio::task::spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).
