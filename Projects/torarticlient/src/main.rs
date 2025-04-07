use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::TlsConnector;
use tokio_native_tls::TlsConnector as TokioTlsConnector;

const DOMAIN: &'static str = "myinstafollow.com";
const PORT: u16 = 443;
const PATH: &'static str = "free-tiktok-views";

#[tokio::main]
pub (crate) async fn main() -> anyhow::Result<()> {
    // Set up native TLS configuration
    let tls_conn = TlsConnector::new()?;
    let tls_conn = TokioTlsConnector::from(tls_conn);

    // Set up Tor client
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;
    
    // Make stream to the target domain with tor
    let stream = client.connect((DOMAIN, PORT)).await?;

    // Wrap the stream with TLS
    let mut stream = tls_conn.connect(DOMAIN, stream).await?;

    // Send HTTP GET request
    let boundary = "----geckoformboundary1537beeb0c2e087b744da49b004303b9";
    let form_data = format!(
        "{boundary}\r\n\
        Content-Disposition: form-data; name=\"service\"\r\n\
        \r\n\
        6473\r\n\
        {boundary}\r\n\
        Content-Disposition: form-data; name=\"postlink\"\r\n\
        \r\n\
        https://www.tiktok.com/@smanike_official/video/7490470891281796359?is_from_webapp=1&sender_device=pc&web_id=7490506889974777351\r\n\
        {boundary}\r\n\
        Content-Disposition: form-data; name=\"tiktokviewsQuantity\"\r\n\
        \r\n\
        300\r\n\
        {boundary}\r\n\
        Content-Disposition: form-data; name=\"extended_user_agent\"\r\n\
        \r\n\
                Browser CodeName: Mozilla | \r\n\
                Browser Name: Netscape | \r\n\
                Browser Version: 5.0 (X11) | \r\n\
                Cookies Enabled: true | \r\n\
                Platform: Linux x86_64 | \r\n\
                User-agent header: Mozilla/5.0 (X11; Linux x86_64; rv:136.0) Gecko/20100101 Firefox/136.0 | \r\n\
                Language: en-US | \r\n\
                Screen Resolution: 1280x800 | \r\n\
                Color Depth: 24 | \r\n\
                Browser Window Size: 1260x271 | \r\n\
                Time Zone: Asia/Jakarta | \r\n\
                Languages: en-US, en | \r\n\
                Hardware Concurrency: 12 | \r\n\
                Device Memory: undefined GB | \r\n\
                Touch Support: false | \r\n\
                JavaScript Enabled: true\r\n\
            \r\n\
        {boundary}--\r\n"
    );

    let request = format!(
        "POST /themes/vision/part/free-tiktok-views/submitForm.php HTTP/1.1\r\n\
        Host: {DOMAIN}\r\n\
        User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:136.0) Gecko/20100101 Firefox/136.0\r\n\
        Accept: */*\r\n\
        Accept-Language: en-US,en;q=0.5\r\n\
        Accept-Encoding: gzip, deflate, br, zstd\r\n\
        Referer: https://myinstafollow.com/free-tiktok-views/\r\n\
        Content-Type: multipart/form-data; boundary={boundary}\r\n\
        Content-Length: {}\r\n\
        Origin: https://myinstafollow.com\r\n\
        DNT: 1\r\n\
        Sec-GPC: 1\r\n\
        Connection: keep-alive\r\n\
        Cookie: twk_uuid_590644a264f23d19a89b007f=%7B%22uuid%22%3A%221.92PoATKhEhN3AplCJrnyL0qYFXTHnSZEhH47VOPSfeLL2L4aXbp6Kxnk2n9TeHhfmnoKBSNkTSL50rmxh0oHDdHIMX0hkqnesuSVKmDOPNCClmu6rw291CeNyZte%22%2C%22version%22%3A3%2C%22domain%22%3A%22myinstafollow.com%22%2C%22ts%22%3A1744019706278%7D; PHPSESSID=3b4f7a08a037b33e6aa03f5ace3175ac; colorMode=sun; TawkConnectionTime=0\r\n\
        Sec-Fetch-Dest: empty\r\n\
        Sec-Fetch-Mode: cors\r\n\
        Sec-Fetch-Site: same-origin\r\n\
        Priority: u=4\r\n\
        \r\n\
        {form_data}",
        form_data.len()
    );
    stream.write_all(request.as_bytes()).await?;

    // Flush the stream to ensure the request is sent
    stream.flush().await?;

    // Read response
    let mut buffah = Vec::new();
    stream.read_to_end(&mut buffah).await?;
    println!("Response: {}", String::from_utf8_lossy(&buffah));
    Ok(())
}
