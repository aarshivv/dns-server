# dns-server
A Dns Server prototype

It uses the root DNS `198.41.0.4` as starting point and resolve the DNS recursively (Root → TLD → Authoritative)

It works similar to other recursive DNS Resolver like Google DNS (8.8.8.8), Cloudflare DNS (1.1.1.1)

- Resolve dns recursively
- Cache the final result

TODO:
- Add expiring of cached result based on ttl
