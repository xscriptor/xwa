use chrono::Utc;

#[derive(Clone)]
pub struct WarcRecord {
    pub target_uri: String,
    pub date: String,
    pub content_type: String,
    pub body: Vec<u8>,
}

impl WarcRecord {
    pub fn new(url: &str, content_type: &str, body: &[u8]) -> Self {
        Self {
            target_uri: url.to_string(),
            date: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            content_type: content_type.to_string(),
            body: body.to_vec(),
        }
    }

    pub fn to_warc_bytes(&self) -> Vec<u8> {
        let http_headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            self.content_type,
            self.body.len()
        );

        let block = [http_headers.as_bytes(), &self.body].concat();
        let content_length = block.len();

        let warc = format!(
            "WARC/1.0\r\n\
             WARC-Type: response\r\n\
             WARC-Date: {}\r\n\
             WARC-Target-URI: {}\r\n\
             Content-Length: {}\r\n\
             \r\n",
            self.date, self.target_uri, content_length
        );

        [warc.as_bytes(), &block, b"\r\n\r\n"].concat()
    }
}

pub fn create_warc_file(records: &[WarcRecord]) -> Vec<u8> {
    let mut data = Vec::new();
    for record in records {
        data.extend_from_slice(&record.to_warc_bytes());
    }
    data
}
