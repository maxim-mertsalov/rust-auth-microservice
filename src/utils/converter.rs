use actix_web::http::header;
use actix_web::HttpRequest;
use aws_lc_rs::digest;
use crate::models::auth::sessions::DeviceInfo;

pub struct Converter;

impl Converter {
    pub fn new_device_info(http_req: HttpRequest) -> DeviceInfo {
        DeviceInfo {
            ip_address: http_req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string(),
            user_agent: http_req.headers().get(header::USER_AGENT).and_then(|h| h.to_str().ok()).unwrap_or("unknown").to_string(),
        }
    }

    pub fn hash_string(input: &String) -> String {
        // let hash_digest = digest::digest(&digest::SHA256, input.as_ref());
        let hash_digest = digest::digest(&digest::SHA256, &input.as_bytes());

        hex::encode(hash_digest.as_ref())
    }
}