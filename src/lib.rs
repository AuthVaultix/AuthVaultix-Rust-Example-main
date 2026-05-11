use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use chrono::{Local, TimeZone};
use base64::{Engine as _, engine::general_purpose};

const BASE_URL: &str = "https://authvaultix.com/api/1.0/";

#[derive(Deserialize, Clone)]
struct ApiResponse<T> {
    success: bool,
    message: Option<String>,
    sessionid: Option<String>,
    info: Option<T>,
}

#[derive(Deserialize, Clone)]
struct BasicResponse {
    success: bool,
    message: Option<String>,
    sessionid: Option<String>,
}

#[derive(Deserialize, Clone)]
struct DataResponse {
    success: bool,
    message: Option<String>,
    contents: Option<String>,
}

#[derive(Deserialize, Clone)]
struct VarResponse {
    success: bool,
    message: Option<String>,
    response: Option<String>,
}

#[derive(Deserialize, Clone)]
struct OnlineResponse {
    success: bool,
    message: Option<String>,
    users: Option<Vec<OnlineUser>>,
}

#[derive(Deserialize, Clone)]
struct ChatResponse {
    success: bool,
    message: Option<String>,
    code: Option<i32>,
    remaining_seconds: Option<i32>,
    muted_until: Option<String>,
    remaining_human: Option<String>,
}

#[derive(Deserialize, Clone)]
struct ChatHistoryResponse {
    success: bool,
    message: Option<String>,
    messages: Option<Vec<ChatMessage>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct UserInfo {
    pub username: String,
    pub ip: Option<String>,
    pub hwid: Option<String>,
    pub createdate: Option<String>,
    pub lastlogin: Option<String>,
    pub subscriptions: Option<Vec<Subscription>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct Subscription {
    pub subscription: String,
    pub key: Option<String>,
    pub expiry: String,
    pub timeleft: u64,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct OnlineUser {
    pub credential: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub author: String,
    pub role: String,
    pub message: String,
    pub timestamp: i64,
}

pub struct NetworkAgent;

impl NetworkAgent {
    fn post<T: for<'de> Deserialize<'de>>(url: &str, payload: &HashMap<&str, String>) -> T {
        let client = Client::builder().build().expect("Failed to build HTTP client");
        let response = client
            .post(url)
            .form(payload)
            .send()
            .expect("Request failed");

        let text = response.text().expect("Failed to read response text");
        serde_json::from_str::<T>(&text)
            .unwrap_or_else(|_| panic!("❌ Invalid JSON from server: {}", text))
    }
}

pub struct PayloadBuilder<'a> {
    payload: HashMap<&'a str, String>,
}

impl<'a> PayloadBuilder<'a> {
    pub fn new(type_val: &str) -> Self {
        let mut pb = Self {
            payload: HashMap::new(),
        };
        pb.payload.insert("type", type_val.to_string());
        pb
    }

    pub fn with_context(mut self, name: &str, ownerid: &str, sessionid: Option<&String>) -> Self {
        self.payload.insert("name", name.to_string());
        self.payload.insert("ownerid", ownerid.to_string());
        if let Some(sid) = sessionid {
            self.payload.insert("sessionid", sid.clone());
        }
        self
    }

    pub fn with_value(mut self, key: &'a str, value: &str) -> Self {
        self.payload.insert(key, value.to_string());
        self
    }

    pub fn compile(self) -> HashMap<&'a str, String> {
        self.payload
    }
}

pub struct AuthVaultixCore {
    name: String,
    ownerid: String,
    secret: String,
    version: String,
    pub sessionid: Option<String>,
    pub current_user: Option<UserInfo>,
    pub initialized: bool,
}

impl AuthVaultixCore {
    pub fn new(name: &str, ownerid: &str, secret: &str, version: &str) -> Self {
        if name.is_empty() || ownerid.is_empty() || secret.is_empty() || version.is_empty() {
            println!("Application not setup correctly.");
            std::process::exit(1);
        }
        Self {
            name: name.to_string(),
            ownerid: ownerid.to_string(),
            secret: secret.to_string(),
            version: version.to_string(),
            sessionid: None,
            current_user: None,
            initialized: false,
        }
    }

    fn ensure_ready(&self) {
        if !self.initialized {
            println!("SDK not initialized. Call init() before using any API.");
            std::process::exit(1);
        }
    }

    pub fn init(&mut self) {
        if self.initialized { return; }
        
        let payload = PayloadBuilder::new("init")
            .with_value("ver", &self.version)
            .with_value("name", &self.name)
            .with_value("ownerid", &self.ownerid)
            .compile();

        let resp: ApiResponse<UserInfo> = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            if let Some(sid) = resp.sessionid {
                self.sessionid = Some(sid);
            }
            self.initialized = true;
            println!("✅ Initialized Successfully! Session ID: {:?}", self.sessionid);
        } else {
            let msg = resp.message.unwrap_or_else(|| "Unknown error".to_string());
            println!("❌ Init failed: {}", msg);
            std::process::exit(1);
        }
    }

    pub fn authenticate_user(&mut self, username: &str, pass: &str) {
        self.ensure_ready();
        let hwid = crate::get_hwid();
        let payload = PayloadBuilder::new("login")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("username", username)
            .with_value("pass", pass)
            .with_value("hwid", &hwid)
            .compile();

        let resp: ApiResponse<UserInfo> = NetworkAgent::post(BASE_URL, &payload);
        self.handle_auth_response(resp, "✅ Logged in!");
    }

    pub fn validate_session(&self) -> bool {
        self.ensure_ready();
        if self.sessionid.is_none() {
            println!("❌ Session missing");
            return false;
        }

        let payload = PayloadBuilder::new("check")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            println!("✅ Session Valid!");
            true
        } else {
            println!("❌ Session Invalid: {:?}", resp.message);
            false
        }
    }

    pub fn register_account(&mut self, username: &str, pass: &str, license: &str, email: &str) {
        self.ensure_ready();
        let hwid = crate::get_hwid();
        let payload = PayloadBuilder::new("register")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("username", username)
            .with_value("pass", pass)
            .with_value("key", license)
            .with_value("email", email)
            .with_value("hwid", &hwid)
            .compile();

        let resp: ApiResponse<UserInfo> = NetworkAgent::post(BASE_URL, &payload);
        self.handle_auth_response(resp, "✅ Registered Successfully!");
    }

    pub fn license_access(&mut self, license: &str) {
        self.ensure_ready();
        let hwid = crate::get_hwid();
        let payload = PayloadBuilder::new("license")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("key", license)
            .with_value("hwid", &hwid)
            .compile();

        let resp: ApiResponse<UserInfo> = NetworkAgent::post(BASE_URL, &payload);
        self.handle_auth_response(resp, "✅ License Login Successful!");
    }

    pub fn send_log(&self, message: &str) -> bool {
        self.ensure_ready();
        let pcuser = std::env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string());
        let payload = PayloadBuilder::new("log")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("message", message)
            .with_value("pcuser", &pcuser)
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            true
        } else {
            println!("❌ Log failed: {:?}", resp.message);
            false
        }
    }

    pub fn retrieve_file(&self, fileid: &str) -> Option<Vec<u8>> {
        self.ensure_ready();
        let payload = PayloadBuilder::new("file")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("fileid", fileid)
            .compile();

        let resp: DataResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            if let Some(contents) = resp.contents {
                if let Ok(bytes) = general_purpose::STANDARD.decode(contents) {
                    println!("✅ Download successful");
                    return Some(bytes);
                }
            }
        }
        println!("❌ Download failed: {:?}", resp.message);
        None
    }

    pub fn get_online_clients(&self) -> Option<Vec<OnlineUser>> {
        self.ensure_ready();
        let payload = PayloadBuilder::new("fetchonline")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .compile();

        let resp: OnlineResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            resp.users
        } else {
            println!("❌ Fetch online users failed: {:?}", resp.message);
            None
        }
    }

    pub fn enforce_ban(&self, reason: &str) -> bool {
        self.ensure_ready();
        let payload = PayloadBuilder::new("ban")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("reason", reason)
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            println!("✅ Banned successfully");
            true
        } else {
            println!("❌ Ban failed: {:?}", resp.message);
            false
        }
    }

    pub fn terminate_session(&mut self) {
        self.ensure_ready();
        let payload = PayloadBuilder::new("logout")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            self.sessionid = None;
            self.initialized = false;
            println!("✅ Logged out successfully");
        } else {
            println!("❌ Logout error: {:?}", resp.message);
        }
    }

    pub fn update_username(&mut self, new_username: &str) {
        self.ensure_ready();
        let payload = PayloadBuilder::new("changeusername")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("newUsername", new_username)
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            self.sessionid = None;
            self.initialized = false;
            println!("✅ Username changed successfully. Please login again.");
        } else {
            println!("❌ Change username error: {:?}", resp.message);
        }
    }

    pub fn verify_blacklist(&self) -> bool {
        self.ensure_ready();
        let hwid = crate::get_hwid();
        let payload = PayloadBuilder::new("checkblacklist")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("hwid", &hwid)
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if !resp.success {
            println!("❌ Client is blacklisted: {:?}", resp.message);
            false
        } else {
            true
        }
    }

    pub fn apply_upgrade(&self, username: &str, license: &str) -> bool {
        self.ensure_ready();
        let payload = PayloadBuilder::new("upgrade")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("username", username)
            .with_value("key", license)
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            println!("✅ Upgrade successful");
            true
        } else {
            let msg = resp.message.unwrap_or_else(|| "Unknown error".to_string());
            println!("❌ Upgrade failed: {}", msg);
            false
        }
    }

    pub fn trigger_password_reset(&self, username: &str, email: &str) -> bool {
        self.ensure_ready();
        let payload = PayloadBuilder::new("forgot")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("username", username)
            .with_value("email", email)
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            println!("✅ Reset email sent successfully");
            true
        } else {
            let msg = resp.message.unwrap_or_else(|| "Unknown error".to_string());
            println!("❌ Password reset failed: {}", msg);
            false
        }
    }

    pub fn fetch_global_variable(&self, varid: &str) -> Option<String> {
        self.ensure_ready();
        let payload = PayloadBuilder::new("var")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("varid", varid)
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            resp.message
        } else {
            println!("❌ Failed to fetch global var: {:?}", resp.message);
            None
        }
    }

    pub fn fetch_user_variable(&self, var_name: &str) -> Option<String> {
        self.ensure_ready();
        let payload = PayloadBuilder::new("getvar")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("var", var_name)
            .compile();

        let resp: VarResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            resp.response
        } else {
            println!("❌ Failed to get user var: {:?}", resp.message);
            None
        }
    }

    pub fn update_user_variable(&self, var_name: &str, value: &str) -> bool {
        self.ensure_ready();
        let payload = PayloadBuilder::new("setvar")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("var", var_name)
            .with_value("data", value)
            .compile();

        let resp: BasicResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            true
        } else {
            println!("❌ Failed to set user var: {:?}", resp.message);
            false
        }
    }

    pub fn transmit_chat_message(&self, message: &str, channel: &str) -> bool {
        self.ensure_ready();
        let payload = PayloadBuilder::new("chatsend")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("message", message)
            .with_value("channel", channel)
            .compile();

        let resp: ChatResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            println!("✅ Message sent!");
            true
        } else {
            if resp.code == Some(403) && resp.remaining_seconds.unwrap_or(0) > 0 {
                println!("❌ Muted till {} (wait {})", resp.muted_until.unwrap_or_default(), resp.remaining_human.unwrap_or_default());
            } else {
                println!("❌ Failed to send chat: {:?}", resp.message);
            }
            false
        }
    }

    pub fn retrieve_chat_history(&self, channel: &str) -> Option<Vec<ChatMessage>> {
        self.ensure_ready();
        let payload = PayloadBuilder::new("chatfetch")
            .with_context(&self.name, &self.ownerid, self.sessionid.as_ref())
            .with_value("channel", channel)
            .compile();

        let resp: ChatHistoryResponse = NetworkAgent::post(BASE_URL, &payload);
        if resp.success {
            resp.messages
        } else {
            println!("❌ Failed to fetch chat history: {:?}", resp.message);
            None
        }
    }

    fn handle_auth_response(&mut self, resp: ApiResponse<UserInfo>, success_msg: &str) {
        if resp.success {
            println!("{}", success_msg);
            if let Some(info) = resp.info {
                self.current_user = Some(info.clone());
                crate::print_user_info(&info);
            }
        } else {
            let msg = resp.message.unwrap_or_else(|| "Unknown error".to_string());
            println!("❌ Error: {}", msg);
        }
    }
}

pub struct AuthVaultix {
    core: AuthVaultixCore,
}

impl AuthVaultix {
    pub fn new(name: &str, ownerid: &str, secret: &str, version: &str) -> Self {
        Self {
            core: AuthVaultixCore::new(name, ownerid, secret, version),
        }
    }

    pub fn init(&mut self) { self.core.init(); }
    pub fn login(&mut self, username: &str, pass: &str) { self.core.authenticate_user(username, pass); }
    pub fn check(&self) -> bool { self.core.validate_session() }
    pub fn register(&mut self, username: &str, pass: &str, license: &str, email: &str) { self.core.register_account(username, pass, license, email); }
    pub fn license_login(&mut self, license: &str) { self.core.license_access(license); }
    pub fn log(&self, message: &str) -> bool { self.core.send_log(message) }
    pub fn download(&self, fileid: &str) -> Option<Vec<u8>> { self.core.retrieve_file(fileid) }
    pub fn fetch_online(&self) -> Option<Vec<OnlineUser>> { self.core.get_online_clients() }
    pub fn ban(&self, reason: &str) -> bool { self.core.enforce_ban(reason) }
    pub fn logout(&mut self) { self.core.terminate_session(); }
    pub fn change_username(&mut self, new_username: &str) { self.core.update_username(new_username); }
    pub fn check_blacklist(&self) -> bool { self.core.verify_blacklist() }
    pub fn upgrade(&self, username: &str, license: &str) -> bool { self.core.apply_upgrade(username, license) }
    pub fn forgot_password(&self, username: &str, email: &str) -> bool { self.core.trigger_password_reset(username, email) }
    pub fn get_global_var(&self, varid: &str) -> Option<String> { self.core.fetch_global_variable(varid) }
    pub fn get_var(&self, var_name: &str) -> Option<String> { self.core.fetch_user_variable(var_name) }
    pub fn set_var(&self, var_name: &str, value: &str) -> bool { self.core.update_user_variable(var_name, value) }
    pub fn chat_send(&self, message: &str, channel: &str) -> bool { self.core.transmit_chat_message(message, channel) }
    pub fn chat_fetch(&self, channel: &str) -> Option<Vec<ChatMessage>> { self.core.retrieve_chat_history(channel) }
}

fn print_user_info(info: &UserInfo) {
    println!("\n=== User Data ===");
    println!("Username: {}", info.username);
    if let Some(ip) = &info.ip { println!("IP: {}", ip); }
    if let Some(hwid) = &info.hwid { println!("HWID: {}", hwid); }

    fn format_date(ts: &str) -> String {
        if let Ok(parsed) = ts.parse::<i64>() {
            if let Some(dt) = chrono::DateTime::from_timestamp(parsed, 0) {
                let local = Local.from_utc_datetime(&dt.naive_utc());
                return local.format("%Y-%m-%d %I:%M:%S %p").to_string();
            }
        }
        ts.to_string()
    }
    
    if let Some(created) = &info.createdate { println!("Created: {}", format_date(created)); }
    if let Some(last) = &info.lastlogin { println!("Last Login: {}", format_date(last)); }

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
                println!("[{}] {} | Expiry: {} | Timeleft: {}", i + 1, s.subscription, expiry, timeleft);
            }
        }
    }
    println!();
}

fn get_hwid() -> String {
    if let Ok(output) = std::process::Command::new("powershell")
        .args(["-Command", "[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value"])
        .output()
    {
        let sid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sid.is_empty() { return sid; }
    }
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
