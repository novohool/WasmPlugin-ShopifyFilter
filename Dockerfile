# 使用 distroless 基础镜像，专门用于 WASM 文件
FROM gcr.io/distroless/static-debian12:nonroot

# 创建 WASM 插件目录
RUN mkdir -p /etc/wasm

# 复制 WASM 文件到 Istio 期望的位置
COPY target/wasm32-wasip1/release/shopify_filter.wasm /etc/wasm/shopify_filter.wasm

# 设置文件权限
USER nonroot

# 设置标签
LABEL org.opencontainers.image.title="Shopify CSP Filter"
LABEL org.opencontainers.image.description="Proxy-WASM plugin for Shopify CSP filtering"
LABEL org.opencontainers.image.version="1.0"
LABEL org.opencontainers.image.source="https://github.com/novohool/WasmPlugin-ShopifyFilter"
