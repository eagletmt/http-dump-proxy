# http-dump-proxy
HTTP reverse proxy with request/response dump for debugging.

## Usage
```
% http-dump-proxy -u https://wanko.cc
2022-08-02T14:31:15.507976Z  INFO http_dump_proxy: Listen 127.0.0.1:8080
```

```
curl http://localhost:8080/
```

```
2022-08-02T14:31:36.454099Z  INFO http_dump_proxy: Received request from downstream method=GET path=/
host: localhost:8080
user-agent: curl/7.84.0
accept: */*

2022-08-02T14:31:36.454685Z  INFO http_dump_proxy: Send request to upstream uri=https://wanko.cc/
2022-08-02T14:31:36.630463Z  INFO http_dump_proxy: Received response from upstream status=200 OK
content-type: text/html
content-length: 5336
connection: keep-alive
last-modified: Tue, 04 Jan 2022 16:44:00 GMT
server: AmazonS3
date: Tue, 02 Aug 2022 14:31:58 GMT
etag: "2736cdd2ca82cd3fe50a731655d4247a"
x-cache: RefreshHit from cloudfront
via: 1.1 983d7210fe21e3eb1ad56033839bd3b2.cloudfront.net (CloudFront)
x-amz-cf-pop: NRT57-C1
x-amz-cf-id: Cw6ygclYXCbP-6qLSQMaZg2CBdI_6ywI8Kk8a6bJFOUQjUmuXsJCJg==

<!DOCTYPE html><html>(snip)</html>
```
