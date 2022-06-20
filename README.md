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


DNS format

| RFC Name | Descriptive Name     | Length             | Description                                                                                                                                    |
| -------- | -------------------- | ------------------ | -----------------------------------------------------------------------------------------------------------------------------------------------|
| ID       | Packet Identifier    | 16 bits            | A random identifier is assigned to query packets. Response packets must reply with the same id.                                                |
| QR       | Query Response       | 1 bit              | 0 for queries, 1 for responses.                                                                                                                |
| OPCODE   | Operation Code       | 4 bits             | Typically always 0, see RFC1035 for details.                                                                                                   |
| AA       | Authoritative Answer | 1 bit              | Set to 1 if the responding server is authoritative - that is, it "owns" - the domain queried.                                                  |
| TC       | Truncated Message    | 1 bit              | Set to 1 if the message length exceeds 512 bytes.                                                                                              |
| RD       | Recursion Desired    | 1 bit              | Set by the sender of the request if the server should attempt to resolve the query recursively if it does not have an answer readily available.|
| RA       | Recursion Available  | 1 bit              | Set by the server to indicate whether or not recursive queries are allowed.                                                                    |
| Z        | Reserved             | 3 bits             | Originally reserved for later use, but now used for DNSSEC queries.                                                                            |
| RCODE    | Response Code        | 4 bits             | Set by the server to indicate the status of the response.                                                                                      |
| QDCOUNT  | Question Count       | 16 bits            | The number of entries in the Question Section                                                                                                  |
| ANCOUNT  | Answer Count         | 16 bits            | The number of entries in the Answer Section                                                                                                    |
| NSCOUNT  | Authority Count      | 16 bits            | The number of entries in the Authority Section                                                                                                 |
| ARCOUNT  | Additional Count     | 16 bits            | The number of entries in the Additional Section    

| Field  | Type           | Description                                                          |
| ------ | -------------- | -------------------------------------------------------------------- |
| Name   | Label Sequence | The domain name, encoded as a sequence of labels as described below. |
| Type   | 2-byte Integer | The record type.                                                     |
| Class  | 2-byte Integer | The class, in practice always set to 1.                              |


| Field  | Type           | Description                                                                       |
| ------ | -------------- | --------------------------------------------------------------------------------- |
| Name   | Label Sequence | The domain name, encoded as a sequence of labels as described below.              |
| Type   | 2-byte Integer | The record type.                                                                  |
| Class  | 2-byte Integer | The class, in practice always set to 1.                                           |
| TTL    | 4-byte Integer | Time-To-Live, i.e. how long a record can be cached before it should be requeried. |
| Len    | 2-byte Integer | Length of the record type specific data.                                          |