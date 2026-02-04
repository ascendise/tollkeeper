# Tollkeeper - the PoW proxy firewall

Tollkeeper allows you to block, or at least rate limit, lazy crawlers and other
bots from accessing your services without affecting benign users.

It achieves this goal by inspecting incoming requests and returning a
[Proof-Of-Work-challenge](http://www.hashcash.org/) to be solved before
allowing access to the service (e.g. a website).

Lazy crawlers will likely not execute javascript or run custom code to solve the
challenge and therefore get locked out.

Users accessing from browsers will only notice their machine executing some
heavy JS for a couple seconds and then their intended page loading.

And for non-interactive services, like Web APIs, the API will return
the challenge as json, allowing you to solve the challenge as part of your workflow.

## How to install (docker)

You can either build the docker image yourself using the `Dockerfile` in this repo
or pull a release from <https://docker.nexus.ascendise.ch/>

To deploy the image, you can use the following `compose.yml` as base.

```yaml
services:
  tollkeeper:
    environment:
      RUST_LOG: info # See https://docs.rs/log/latest/log/enum.Level.html
      container_name: tollkeeper
      image: docker.ascendise.ch/ascendise/tollkeeper:latest  # NOTE: latest is unstable
      ports:
        - 8000:8000 # Proxy Socket
        - 8080:8080 # API Socket (Payment API, static assets)
      volumes:
        - ./config.toml:/usr/local/bin/app/config.toml # tollkeeper configuration
```

Next thing to do is to set up your network to route requests for services
to protect to tollkeeper.
E.g. in caddy this could look something like this

```Caddyfile
yourservice.example.ch {
  reverse_proxy tollkeeper:8000 {
    header_up X-Real-Ip {remote_host}
  }
}
```

> NOTE: When tollkeeper is deployed behind a reverse proxy, you should set a header
containing the remote client ip and tell tollkeeper about it, or else you will only
guard your service from your proxy ;D

You must expose both tollkeeper ports, best as separate domains.
E.g.

```Caddyfile
tollkeeper.example.ch {
  reverse_proxy tollkeeper:8080 {
    header_up X-Real-Ip {remote_host}
  }
}
```

> NOTE: API also requires the same "X-Real-IP" header

## Configuration

Tollkeepers configuration is stored inside a `config.toml` file.

```toml
# Provide a key for signing Tokens/Challenges
secret_key_provider = { InMemory = "snowball2" }

[server]
# Optional
proxy_port = 8000
# Optional
api_port = 8080

[api]
# Public URL for accessing the Tollkeeper API
# This should be the URL clients should send their payments to
base_url = "https://tollkeeper.example.ch/"
# (Optional) if tollkeeper is deployed behind a proxy.
# Must contain the IP of the client trying to access your website and be provided
# to both API (8080) and Proxy Socket (8000)
real_ip_header = "X-Real-Ip" 

# Gates define all services you want to protect.
[gates]

[gates.example_gate] 
# Destination to guard. Must be a valid url (HTTPS is not supported)
destination = "http://yourservice.example.ch/" 
# (Optional) Points to the internal url of your service. 
# This is the url tollkeeper actually connects to. If omitted, tollkeeper
# will connect to the specified destination, which might cause a loop depending
# on setup
internal_destination = "http://yourservice:9000/"
# Orders tollkeeper goes through for incoming requests to this service
# _Order_-dependant. Tollkeeper will handle a request with the first matching order
orders = [ "debug_order", "hash_cash_order" ]

# This order will trigger on requests containing the query parameter `?debug`
# This allows you to view the beautiful challenge page and maybe even solve a
# difficult challenge?
[orders.debug_order]
# Check request target for query param
descriptions = [{Regex = {key = "destination", regex = "\\?debug"}}]
# Denies access (sends challenge) if any descriptions match
access_policy = "Blacklist"
# Return a hashcash challenge with difficulty 99 
toll_declaration = { Hashcash = {expiry = "1h", difficulty = 99}}

# If the debug_order does not trigger, we check with the last order
# all other cases. We allow a few _descriptions_ and challenge the rest
[orders.hash_cash_order]
descriptions = [
  # Check if request comes from curl
 {Regex = {key = "user_agent", regex = "curl"}},
  # Check if request is going to /api
 {Regex = {key = "destination", regex = "yourservice.example.ch:80/api/"}}
]
# Allows access (proxies request to target) if any descriptions match
# Else returns a challenge as it is the last order
access_policy = "Whitelist"
# Return a hashcash challenge with saner difficulty
toll_declaration = { Hashcash = {expiry = "30m", difficulty = 12}}
```
