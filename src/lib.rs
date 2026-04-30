use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use chrono::{Local, TimeZone};

const BASE_URL: &str = "https://authvaultix.com/api/1.0/";

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    message: Option<String>,
    sessionid: Option<String>,
    info: Option<T>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct UserInfo {
    username: String,
    ip: Option<String>,
    hwid: Option<String>,
    createdate: Option<String>,
    lastlogin: Option<String>,
    subscriptions: Option<Vec<Subscription>>,
}


#[allow(dead_code)]
#[derive(Deserialize)]
struct Subscription {
    subscription: String,
    key: Option<String>,
    expiry: String,
    timeleft: u64,
}


pub struct AuthVaultix {
    name: String,
    ownerid: String,
    secret: String,
    version: String,
    sessionid: Option<String>,
}

impl AuthVaultix {
    pub fn new(name: &str, ownerid: &str, secret: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            ownerid: ownerid.to_string(),
            secret: secret.to_string(),
            version: version.to_string(),
            sessionid: None,
        }
    }

    fn client(&self) -> Client {
        Client::builder().build().expect("Failed to build HTTP client")
    }

    fn send_request<T: for<'de> Deserialize<'de>>(
        &self,
        payload: HashMap<&str, String>,
    ) -> ApiResponse<T> {
        let client = self.client();
        let response = client
            .post(BASE_URL)
            .form(&payload)
            .send()
            .expect("Request failed");

        let text = response.text().expect("Failed to read response text");
        serde_json::from_str::<ApiResponse<T>>(&text)
            .unwrap_or_else(|_| panic!("❌ Invalid JSON from server: {}", text))
    }

    pub fn init(&mut self) {
        let mut payload = HashMap::new();
        payload.insert("type", "init".to_string());
        payload.insert("name", self.name.clone());
        payload.insert("ownerid", self.ownerid.clone());
        payload.insert("secret", self.secret.clone());
        payload.insert("ver", self.version.clone());

        let resp: ApiResponse<UserInfo> = self.send_request(payload);
        if resp.success {
            if let Some(sid) = resp.sessionid {
                self.sessionid = Some(sid);
            }
            println!("✅ Initialized Successfully!");
        } else {
            let msg = resp.message.unwrap_or_else(|| "Unknown error".to_string());
            println!("❌ Init failed: {}", msg);
            std::process::exit(1);
        }
    }

    pub fn login(&self, username: &str, pass: &str) {
        let sid = self.sessionid.as_ref().expect("App not initialized. Run init() first.");
        let mut payload = HashMap::new();
        payload.insert("type", "login".to_string());
        payload.insert("sessionid", sid.clone());
        payload.insert("username", username.to_string());
        payload.insert("pass", pass.to_string());
        payload.insert("hwid", Self::get_hwid());
        payload.insert("name", self.name.clone());
        payload.insert("ownerid", self.ownerid.clone());

        let resp: ApiResponse<UserInfo> = self.send_request(payload);
        Self::handle_auth_response(resp, "✅ Logged in!");
    }

    pub fn register(&self, username: &str, pass: &str, license: &str) {
        let sid = self.sessionid.as_ref().expect("App not initialized. Run init() first.");
        let mut payload = HashMap::new();
        payload.insert("type", "register".to_string());
        payload.insert("sessionid", sid.clone());
        payload.insert("username", username.to_string());
        payload.insert("pass", pass.to_string());
        payload.insert("key", license.to_string());
        payload.insert("hwid", Self::get_hwid());
        payload.insert("name", self.name.clone());
        payload.insert("ownerid", self.ownerid.clone());

        let resp: ApiResponse<UserInfo> = self.send_request(payload);
        Self::handle_auth_response(resp, "✅ Registered Successfully!");
    }

    pub fn license_login(&self, license: &str) {
        let sid = self.sessionid.as_ref().expect("App not initialized. Run init() first.");
        let mut payload = HashMap::new();
        payload.insert("type", "license".to_string());
        payload.insert("sessionid", sid.clone());
        payload.insert("key", license.to_string());
        payload.insert("hwid", Self::get_hwid());
        payload.insert("name", self.name.clone());
        payload.insert("ownerid", self.ownerid.clone());

        let resp: ApiResponse<UserInfo> = self.send_request(payload);
        Self::handle_auth_response(resp, "✅ License Login Successful!");
    }

    fn handle_auth_response(resp: ApiResponse<UserInfo>, success_msg: &str) {
        if resp.success {
            println!("{}", success_msg);
            if let Some(info) = resp.info {
                Self::print_user_info(&info);
            }
        } else {
            let msg = resp.message.unwrap_or_else(|| "Unknown error".to_string());
            println!("❌ Error: {}", msg);
        }
    }

fn print_user_info(info: &UserInfo) {
    println!("\n=== User Data ===");

    println!("Username: {}", info.username);

    if let Some(ip) = &info.ip {
        println!("IP: {}", ip);
    }

    if let Some(hwid) = &info.hwid {
        println!("HWID: {}", hwid);
    }

    // 🕒 safe string → date
    

    fn format_date(ts: &str) -> String {
        if let Ok(parsed) = ts.parse::<i64>() {
            if let Some(dt) = chrono::DateTime::from_timestamp(parsed, 0) {
                let local = Local.from_utc_datetime(&dt.naive_utc());
                return local.format("%Y-%m-%d %I:%M:%S %p").to_string();
            }
        }
        ts.to_string()
    }
    if let Some(created) = &info.createdate {
        println!("Created: {}", format_date(created));
    }

    if let Some(last) = &info.lastlogin {
        println!("Last Login: {}", format_date(last));
    }

    // ⏳ timeleft formatter
    fn format_timeleft(seconds: u64) -> String {
        let d = seconds / 86400;
        let h = (seconds % 86400) / 3600;
        let m = (seconds % 3600) / 60;
        format!("{}d {}h {}m", d, h, m)
    }

    if let Some(subs) = &info.subscriptions {
        if !subs.is_empty() {
            println!("\nSubscriptions:");

            for (i, s) in subs.iter().enumerate() {
                let expiry = format_date(&s.expiry);
                let timeleft = format_timeleft(s.timeleft);

                println!(
                    "[{}] {} | Expiry: {} | Timeleft: {}",
                    i + 1,
                    s.subscription,
                    expiry,
                    timeleft
                );
            }
        }
    }

    println!();
}

fn get_hwid() -> String {

    if let Ok(output) = std::process::Command::new("powershell")
        .args([
            "-Command",
            "[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value",
        ])
        .output()
    {
        let sid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sid.is_empty() {
            return sid;
        }
    }

    // Fallback: legacy WMIC (CMD)
    if let Ok(output) = std::process::Command::new("cmd")
        .args(["/C", "wmic useraccount where name='%USERNAME%' get sid /value"])
        .output()
    {
        let out = String::from_utf8_lossy(&output.stdout);
        for line in out.lines() {
            if line.trim().starts_with("SID=") {
                return line.trim().replace("SID=", "").to_string();
            }
        }
    }

    "UNKNOWN_HWID".to_string()
}



}
