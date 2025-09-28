## Proxy-Wasm plugin example: HTTP headers

Proxy-Wasm plugin that logs HTTP request/response headers.

### Building

```sh
$ cargo build --target wasm32-wasip1 --release
```

### Using in Envoy

This example can be run with [`docker compose`](https://docs.docker.com/compose/install/)
and has a matching Envoy configuration.

```sh
$ docker compose up
```

Send HTTP request to `localhost:10000/hello`:

```sh
$ curl -I  localhost:10000/hello?shop=efitteststore.myshopify.com
HTTP/1.1 200 OK
content-length: 32
content-type: text/plain
content-security-policy: block-all-mixed-content; frame-ancestors https://efitteststore.myshopify.com https://admin.shopify.com
date: Sun, 28 Sep 2025 04:08:35 GMT
server: envoy
```

Expected Envoy logs:

```console
[...] wasm log http_headers: #2 -> :authority: localhost:10000
[...] wasm log http_headers: #2 -> :path: /hello
[...] wasm log http_headers: #2 -> :method: GET
[...] wasm log http_headers: #2 -> :scheme: http
[...] wasm log http_headers: #2 -> user-agent: curl/7.81.0
[...] wasm log http_headers: #2 -> accept: */*
[...] wasm log http_headers: #2 -> x-forwarded-proto: http
[...] wasm log http_headers: #2 -> x-request-id: 3ed6eb3b-ddce-4fdc-8862-ddb8f168d406
[...] wasm log http_headers: #2 <- :status: 200
[...] wasm log http_headers: #2 <- hello: World
[...] wasm log http_headers: #2 <- powered-by: proxy-wasm
[...] wasm log http_headers: #2 <- content-length: 14
[...] wasm log http_headers: #2 <- content-type: text/plain
[...] wasm log http_headers: #2 completed.
```


## 打包:
#### 步骤 2：打包oci
```
docker build -t oci://your-registry/shopify-csp:1.0 .
```

#### 步骤 3：创建 WasmPlugin 资源
```
apiVersion: extensions.istio.io/v1alpha1
kind: WasmPlugin
metadata:
  name: shopify-header-csp
  namespace: istio-system
spec:
  selector:
    matchLabels:
      istio: ingressgateway
  url: oci://your-registry/shopify-csp:1.0
  # Istio 会自动在镜像的 /etc/wasm/ 目录下查找 .wasm 文件
  # 或者可以显式指定路径：
  # url: oci://your-registry/shopify-csp:1.0/etc/wasm/shopify_filter.wasm
  priority: 100
```


