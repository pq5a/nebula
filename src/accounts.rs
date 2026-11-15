// Saved Discord accounts: a display name, the login email and the password.
// The list is stored in %APPDATA%\Nebula\accounts.dat, encrypted with Windows
// DPAPI so only the same Windows user can read it. Passwords are never sent
// back to the page. The page only receives them in a switch, which fills in
// Discord's own login form.

use std::{
    fs, io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{json, Value};

pub const FILE_NAME: &str = "accounts.dat";
const MAX_NAME_CHARS: usize = 32;
const MAX_EMAIL_CHARS: usize = 254;
const MAX_PASSWORD_CHARS: usize = 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: String,
}

// Messages from the Discord page (src/chrome.js), sent as JSON with an "op".
#[derive(Debug, PartialEq)]
pub enum Request {
    List,
    Add {
        name: String,
        email: String,
        password: String,
    },
    Remove {
        id: String,
    },
    Switch {
        id: String,
    },
    // Sent by the page when Discord's login form is showing.
    LoginPage,
}

pub fn parse_request(msg: &str) -> Option<Request> {
    let v: Value = serde_json::from_str(msg).ok()?;
    let field = |key: &str| v.get(key).and_then(Value::as_str).map(str::to_owned);
    match v.get("op")?.as_str()? {
        "list" => Some(Request::List),
        "add" => Some(Request::Add {
            name: field("name")?,
            email: field("email")?,
            password: field("password")?,
        }),
        "remove" => Some(Request::Remove { id: field("id")? }),
        "switch" => Some(Request::Switch { id: field("id")? }),
        "login-page" => Some(Request::LoginPage),
        _ => None,
    }
}

// Adds an account, or updates it if the email is already saved. Returns the id.
pub fn add(
    accounts: &mut Vec<Account>,
    name: &str,
    email: &str,
    password: &str,
) -> Result<String, &'static str> {
    let name = name.trim();
    let email = email.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err("Name must be 1 to 32 characters.");
    }
    if email.is_empty() || email.chars().count() > MAX_EMAIL_CHARS || !email.contains('@') {
        return Err("Enter the email address for this account.");
    }
    if password.is_empty() || password.chars().count() > MAX_PASSWORD_CHARS {
        return Err("Enter the password for this account.");
    }
    if let Some(existing) = accounts
        .iter_mut()
        .find(|a| a.email.eq_ignore_ascii_case(email))
    {
        existing.name = name.to_owned();
        existing.email = email.to_owned();
        existing.password = password.to_owned();
        return Ok(existing.id.clone());
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let id = format!("{nanos:x}-{}", accounts.len());
    accounts.push(Account {
        id: id.clone(),
        name: name.to_owned(),
        email: email.to_owned(),
        password: password.to_owned(),
    });
    Ok(id)
}

pub fn remove(accounts: &mut Vec<Account>, id: &str) -> bool {
    let before = accounts.len();
    accounts.retain(|a| a.id != id);
    accounts.len() != before
}

// The account list as JSON without passwords, for the page.
pub fn public_json(accounts: &[Account]) -> String {
    let list: Vec<Value> = accounts
        .iter()
        .map(|a| json!({ "id": a.id, "name": a.name, "email": a.email }))
        .collect();
    Value::Array(list).to_string()
}

pub fn load(path: &Path) -> Vec<Account> {
    let Ok(blob) = fs::read(path) else {
        return Vec::new();
    };
    let Some(plain) = dpapi::unprotect(&blob) else {
        return Vec::new();
    };
    let Ok(Value::Array(items)) = serde_json::from_slice::<Value>(&plain) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            Some(Account {
                id: item.get("id")?.as_str()?.to_owned(),
                name: item.get("name")?.as_str()?.to_owned(),
                email: item.get("email")?.as_str()?.to_owned(),
                password: item.get("password")?.as_str()?.to_owned(),
            })
        })
        .collect()
}

pub fn save(path: &Path, accounts: &[Account]) -> io::Result<()> {
    let list: Vec<Value> = accounts
        .iter()
        .map(|a| json!({ "id": a.id, "name": a.name, "email": a.email, "password": a.password }))
        .collect();
    let plain = serde_json::to_vec(&Value::Array(list))?;
    // Refuse to write tokens unencrypted.
    let blob = dpapi::protect(&plain)
        .ok_or_else(|| io::Error::other("could not encrypt the account list"))?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, blob)?;
    fs::rename(&tmp, path)
}

#[cfg(windows)]
mod dpapi {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
        },
    };

    pub fn protect(data: &[u8]) -> Option<Vec<u8>> {
        run(data, true)
    }

    pub fn unprotect(data: &[u8]) -> Option<Vec<u8>> {
        run(data, false)
    }

    // Encrypts or decrypts for the current Windows user, with no UI prompt.
    fn run(data: &[u8], protect: bool) -> Option<Vec<u8>> {
        let input = CRYPT_INTEGER_BLOB {
            cbData: u32::try_from(data.len()).ok()?,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };
        let ok = unsafe {
            if protect {
                CryptProtectData(
                    &input,
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            } else {
                CryptUnprotectData(
                    &input,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            }
        };
        if ok == 0 || output.pbData.is_null() {
            return None;
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
        unsafe { LocalFree(output.pbData.cast()) };
        Some(bytes)
    }
}

// Without DPAPI nothing is stored, so tokens are never written in plain text.
#[cfg(not(windows))]
mod dpapi {
    pub fn protect(_: &[u8]) -> Option<Vec<u8>> {
        None
    }

    pub fn unprotect(_: &[u8]) -> Option<Vec<u8>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_requests_only() {
        assert_eq!(parse_request(r#"{"op":"list"}"#), Some(Request::List));
        assert_eq!(parse_request(r#"{"op":"login-page"}"#), Some(Request::LoginPage));
        assert_eq!(
            parse_request(r#"{"op":"remove","id":"a"}"#),
            Some(Request::Remove { id: "a".into() })
        );
        assert_eq!(
            parse_request(r#"{"op":"switch","id":"a"}"#),
            Some(Request::Switch { id: "a".into() })
        );
        assert_eq!(parse_request(r#"{"op":"add","name":"x"}"#), None);
        assert_eq!(parse_request(r#"{"op":"exec"}"#), None);
        assert_eq!(parse_request("drag"), None);
        assert_eq!(parse_request("theme:ember"), None);
    }

    #[test]
    fn add_validates_fields() {
        let mut list = Vec::new();
        assert!(add(&mut list, "  ", "a@b.c", "pw").is_err());
        assert!(add(&mut list, &"x".repeat(33), "a@b.c", "pw").is_err());
        assert!(add(&mut list, "main", "not-an-email", "pw").is_err());
        assert!(add(&mut list, "main", "a@b.c", "").is_err());
        assert!(list.is_empty());
        assert!(add(&mut list, "main", "a@b.c", "pw").is_ok());
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn same_email_updates_instead_of_duplicating() {
        let mut list = Vec::new();
        let first = add(&mut list, "one", "a@b.c", "old").unwrap();
        let again = add(&mut list, "renamed", "A@B.C", "new").unwrap();
        assert_eq!(first, again);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "renamed");
        assert_eq!(list[0].password, "new");
    }

    #[test]
    fn public_list_never_includes_passwords() {
        let mut list = Vec::new();
        add(&mut list, "main", "a@b.c", "hunter2-secret").unwrap();
        let json = public_json(&list);
        assert!(json.contains("main"));
        assert!(json.contains("a@b.c"));
        assert!(!json.contains("hunter2-secret"));
    }

    #[test]
    fn remove_by_id() {
        let mut list = Vec::new();
        let id = add(&mut list, "main", "a@b.c", "pw").unwrap();
        assert!(!remove(&mut list, "missing"));
        assert!(remove(&mut list, &id));
        assert!(list.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn stored_file_round_trips_and_is_not_plain_text() {
        let dir = std::env::temp_dir().join(format!("nebula-test-{}", std::process::id()));
        let path = dir.join(FILE_NAME);
        let mut list = Vec::new();
        add(&mut list, "main", "a@b.c", "hunter2-secret").unwrap();
        save(&path, &list).unwrap();

        let raw = fs::read(&path).unwrap();
        assert!(!String::from_utf8_lossy(&raw).contains("hunter2-secret"));
        assert_eq!(load(&path), list);

        let _ = fs::remove_dir_all(&dir);
    }
}
