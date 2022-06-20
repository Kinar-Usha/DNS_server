# DNS Server

The server houses a stub resolver, DNS recursive resolver.
It is compatible with existing DNS query tool dig.

# Reference
[RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034)
[RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035)



## Quick start
1. You will need to have Rust installed. Download [rustup](https://rustup.rs).
2. clone the repo.
3. run `cargo build` && `cargo run`
3. run ```dig @127.0.0.1 -p 2053 google.com``` (if your are using powershell ```dig "@127.0.0.1" -p 2053 google.com```)

## Example
```text
-> dig "@127.0.0.1" -p 2053 twitch.tv                                                                                                                                     
; <<>> DiG 9.16.26 <<>> @127.0.0.1 -p 2053 twitch.tv
; (1 server found)
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 53972
;; flags: qr rd ra; QUERY: 1, ANSWER: 4, AUTHORITY: 4, ADDITIONAL: 4

;; QUESTION SECTION:
;twitch.tv.                     IN      A

;; ANSWER SECTION:
twitch.tv.              3600    IN      A       151.101.130.167
twitch.tv.              3600    IN      A       151.101.66.167
twitch.tv.              3600    IN      A       151.101.2.167
twitch.tv.              3600    IN      A       151.101.194.167

;; AUTHORITY SECTION:
twitch.tv.              172800  IN      NS      ns-1450.awsdns-53.org.
twitch.tv.              172800  IN      NS      ns-1778.awsdns-30.co.uk.
twitch.tv.              172800  IN      NS      ns-219.awsdns-27.com.
twitch.tv.              172800  IN      NS      ns-664.awsdns-19.net.

;; Query time: 500 msec
;; SERVER: 127.0.0.1#2053(127.0.0.1)
;; WHEN: Mon May 16 15:46:05 India Standard Time 2022
;; MSG SIZE  rcvd: 303
```

## Server side
```text
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target\debug\dns_server.exe`
Query = DnsQuestion { name: "twitch.tv", qtype: A }
Looking up A twitch.tv  with ns 198.41.0.4 
Looking up A twitch.tv  with ns 192.42.173.30 
Looking up A ns-219.awsdns-27.com  with ns 198.41.0.4 
Looking up A ns-219.awsdns-27.com  with ns 192.5.6.30 
Looking up A ns-219.awsdns-27.com  with ns 205.251.192.28 
Looking up A twitch.tv  with ns 205.251.192.219 
Answer: A { domain: "twitch.tv", addr: 151.101.130.167, ttl: 3600 } 
Answer: A { domain: "twitch.tv", addr: 151.101.66.167, ttl: 3600 } 
Answer: A { domain: "twitch.tv", addr: 151.101.2.167, ttl: 3600 }
Answer: A { domain: "twitch.tv", addr: 151.101.194.167, ttl: 3600 }
Authorities : NS { domain: "twitch.tv", host: "ns-1450.awsdns-53.org", ttl: 172800 }
Authorities : NS { domain: "twitch.tv", host: "ns-1778.awsdns-30.co.uk", ttl: 172800 }
Authorities : NS { domain: "twitch.tv", host: "ns-219.awsdns-27.com", ttl: 172800 }
Authorities : NS { domain: "twitch.tv", host: "ns-664.awsdns-19.net", ttl: 172800 }
Resource: UNKNOWN { domain: "", qtype: 0, data_len: 0, ttl: 0 }
Resource: UNKNOWN { domain: "", qtype: 0, data_len: 0, ttl: 0 }
Resource: UNKNOWN { domain: "", qtype: 0, data_len: 0, ttl: 0 }
Resource: UNKNOWN { domain: "", qtype: 0, data_len: 0, ttl: 0 }
Skipping record: UNKNOWN { domain: "", qtype: 0, data_len: 0, ttl: 0 }
Skipping record: UNKNOWN { domain: "", qtype: 0, data_len: 0, ttl: 0 }
Skipping record: UNKNOWN { domain: "", qtype: 0, data_len: 0, ttl: 0 }
Skipping record: UNKNOWN { domain: "", qtype: 0, data_len: 0, ttl: 0 } 
```

## Documentation
run `cargo doc --open`
