use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use percent_encoding::percent_decode_str;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| Box::new(HttpHeadersRoot));
    proxy_wasm::set_http_context(|_ctx_id, _root_id| Box::new(HttpHeadersRoot));
}

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}
impl RootContext for HttpHeadersRoot {}
impl HttpContext for HttpHeadersRoot {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        if let Some(path) = self.get_http_request_header(":path") {
            info!("Request path: {}", path);
            let shop = parse_query_param(&path, "shop");
            let shop_url = parse_query_param(&path, "shopUrl");
            if let Some(s) = shop {
                info!("Parsed shop parameter: {}", s);
                let _ = self.set_shared_data("shop", Some(s.as_bytes()), None);
            }
            if let Some(s) = shop_url {
                info!("Parsed shopUrl parameter: {}", s);
                let _ = self.set_shared_data("apipathshop", Some(s.as_bytes()), None);
            }
        }

        if let Some(referer) = self.get_http_request_header("referer") {
            info!("Referer header: {}", referer);
            let refershop = parse_query_param(&referer, "shop");
            if let Some(s) = refershop {
                info!("Parsed shop parameter from referer: {}", s);
                let _ = self.set_shared_data("refershop", Some(s.as_bytes()), None);
            }
        }
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        // 移除 X-Frame-Options
        self.remove_http_response_header("X-Frame-Options");

        let csp_base = "block-all-mixed-content; frame-ancestors".to_string();

        let shop = self.get_shared_string("shop");
        let refershop = self.get_shared_string("refershop");
        let apipathshop = self.get_shared_string("apipathshop");

        let mut domains = Vec::new();
        if let Some(s) = shop {
            let shop_url = if s.starts_with("https://") {
                s
            } else {
                format!("https://{}", s)
            };
            info!("Adding shop domain to CSP: {}", shop_url);
            domains.push(shop_url);
        }
        if let Some(s) = refershop {
            let shop_url = if s.starts_with("https://") {
                s
            } else {
                format!("https://{}", s)
            };
            info!("Adding referer shop domain to CSP: {}", shop_url);
            domains.push(shop_url);
        }
        if let Some(s) = apipathshop {
            let shop_url = if s.starts_with("https://") {
                s
            } else {
                format!("https://{}", s)
            };
            info!("Adding API path shop domain to CSP: {}", shop_url);
            domains.push(shop_url);
        }
        // 固定加入这个域名
        domains.push("https://admin.shopify.com".to_string());

        // 去重
        let unique_domains: Vec<String> =
            domains.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
        let frame_ancestors = unique_domains.join(" ");
        let csp = format!("{} {}", csp_base, frame_ancestors);
        info!("Constructed CSP: {}", csp);

        self.add_http_response_header("content-security-policy", &csp);
        Action::Continue
    }
}

impl HttpHeadersRoot {
    /// 从 shared data 中读取 key 的值（字符串形式），若不存在或出错返回 None
    fn get_shared_string(&mut self, key: &str) -> Option<String> {
        let (opt_bytes, _cas) = self.get_shared_data(key);
        if let Some(bytes) = opt_bytes {
            match String::from_utf8(bytes) {
                Ok(s) => Some(s),
                Err(e) => {
                    info!("Invalid UTF-8 in shared data for key '{}': {:?}", key, e);
                    None
                }
            }
        } else {
            None
        }
    }
}

/// 解析 URL 或 referer 中的查询参数 key
fn parse_query_param(url_str: &str, key: &str) -> Option<String> {
    if let Some((_, qry_and_more)) = url_str.split_once('?') {
        let query = qry_and_more.split('#').next().unwrap_or(qry_and_more);
        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                if k == key {
                    return urldecode(v).ok();
                }
            }
        }
    }
    None
}

/// URL 解码函数，返回 Result<String, Utf8Error>
fn urldecode(input: &str) -> Result<String, std::str::Utf8Error> {
    percent_decode_str(input)
        .decode_utf8()
        .map(|cow| cow.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urldecode() {
        assert_eq!(urldecode("hello%20world").unwrap(), "hello world");
        assert_eq!(urldecode("1%2B2%3D3").unwrap(), "1+2=3");
        assert_eq!(urldecode("foo%5Bbar%5D").unwrap(), "foo[bar]");
        assert_eq!(urldecode("https%3A%2F%2Fexample.com").unwrap(), "https://example.com");
        assert_eq!(urldecode("%E4%B8%AD%E6%96%87").unwrap(), "中文");
        assert_eq!(urldecode("").unwrap(), "");
        assert_eq!(urldecode("normal_string").unwrap(), "normal_string");
    }
}
