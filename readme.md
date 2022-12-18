# mega-get-reqwest-rs

Program to perform repeated GET requests in batches

## Help

```bash
Usage: mega-get-reqwest-rs [OPTIONS] <URL>

Arguments:
  <URL>  HTTP URL

Options:
  -w, --world <WORLD>        Number of threads [default: 4]
  -r, --requests <REQUESTS>  Total number of requests [default: 100]
  -p, --proxy <PROXY>        Proxy URL, e.g. `socks5h://localhost:9050`. For available proxy schemes see `impl ProxyScheme` in https://docs.rs/reqwest/latest/src/reqwest/proxy.rs.html
  -h, --help                 Print help information
  -V, --version              Print version information
```
