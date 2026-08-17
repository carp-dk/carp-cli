# carp-client

A client for the [Copenhagen Research Platform][carp] web service: which
deployment to address, how to hold a session against it, and one function per
documented API operation.

```rust
use std::sync::Arc;

use carp_client::api::endpoints::studies;
use carp_client::{Authenticator, CarpClient, Config, Environment};

let config = Config::for_environment(Environment::Test)?;
let auth = Arc::new(Authenticator::new(&config)?);
auth.ensure_session(|url| println!("Opening {url}")).await?;

let client = CarpClient::new(&config, auth)?;
for study in studies::list(&client).await? {
    println!("{}", study.name);
}
```

- **`config`** — the deployments known by name, the local paths, and the
  precedence between flags, environment and `.env`. Sessions and caches are
  keyed by host, so several deployments can be used side by side.
- **`auth`** — OAuth2 authorization code + PKCE against the CARP Keycloak
  realm, refreshed transparently. The browser flow reports its URL through a
  callback rather than printing it, so a caller decides where that goes.
- **`api`** — typed payloads and one function per operation, including the
  data-stream reads that return a study's measurements.
- **`transfer`** — streaming an export or a study file to disk, reporting
  progress through a closure.

Every model is lenient: a field a deployment does not send, or one added by a
newer CARP, cannot fail a whole response.

No dependency on a terminal. This is what the `carp` command line and the
`carp-cli` Python module are both built on — see
[carp-cli](https://github.com/carp-dk/carp-cli).

[carp]: https://carp.dk
