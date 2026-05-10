use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Headers, Request, RequestInit, Response};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: i64,
    username: String,
    email: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct AuthResponse {
    token: String,
    user: User,
}

#[derive(Serialize, Deserialize, Clone)]
struct Post {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: String,
    updated_at: String,
}

#[wasm_bindgen]
pub struct BlogApp {
    base_url: String,
    token: Option<String>,
    username: Option<String>,
    user_id: Option<i64>,
}

#[wasm_bindgen]
impl BlogApp {
    pub fn new(base_url: String) -> BlogApp {
        let token = Self::load_token_from_storage();
        let username = Self::load_username_from_storage();
        let user_id = Self::load_user_id_from_storage();
        BlogApp {
            base_url,
            token,
            username,
            user_id,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub fn get_username(&self) -> String {
        self.username.clone().unwrap_or_default()
    }

    pub fn get_user_id(&self) -> i64 {
        self.user_id.unwrap_or(0)
    }

    fn save_token(&mut self, token: String, username: String, user_id: i64) {
        self.token = Some(token.clone());
        self.username = Some(username.clone());
        self.user_id = Some(user_id);
        Self::save_token_to_storage(&token);
        Self::save_username_to_storage(&username);
        Self::save_user_id_to_storage(user_id);
    }

    pub fn logout(&mut self) {
        self.token = None;
        self.username = None;
        self.user_id = None;
        Self::clear_storage();
    }

    fn save_token_to_storage(token: &str) {
        if let Some(win) = window() {
            if let Ok(Some(local)) = win.local_storage() {
                let _ = local.set_item("blog_token", token);
            }
        }
    }

    fn load_token_from_storage() -> Option<String> {
        if let Some(win) = window() {
            if let Ok(Some(local)) = win.local_storage() {
                return local.get_item("blog_token").ok().flatten();
            }
        }
        None
    }

    fn save_username_to_storage(username: &str) {
        if let Some(win) = window() {
            if let Ok(Some(local)) = win.local_storage() {
                let _ = local.set_item("blog_username", username);
            }
        }
    }

    fn load_username_from_storage() -> Option<String> {
        if let Some(win) = window() {
            if let Ok(Some(local)) = win.local_storage() {
                return local.get_item("blog_username").ok().flatten();
            }
        }
        None
    }

    fn save_user_id_to_storage(user_id: i64) {
        if let Some(win) = window() {
            if let Ok(Some(local)) = win.local_storage() {
                let _ = local.set_item("blog_user_id", &user_id.to_string());
            }
        }
    }

    fn load_user_id_from_storage() -> Option<i64> {
        if let Some(win) = window() {
            if let Ok(Some(local)) = win.local_storage() {
                if let Ok(Some(val)) = local.get_item("blog_user_id") {
                    return val.parse().ok();
                }
            }
        }
        None
    }

    fn clear_storage() {
        if let Some(win) = window() {
            if let Ok(Some(local)) = win.local_storage() {
                let _ = local.remove_item("blog_token");
                let _ = local.remove_item("blog_username");
                let _ = local.remove_item("blog_user_id");
            }
        }
    }

    /// request выполняет HTTP-запрос к серверу.
    /// - `method`: HTTP метод (GET, POST, PUT, DELETE).
    /// - `path`: путь относительно base_url.
    /// - `body`: опциональное тело запроса (сериализуется в JSON).
    /// Return:
    /// - `JsValue` с распарсенным JSON-ответом или ошибку.
    async fn request<T: Serialize + ?Sized>(
        &self,
        method: &str,
        path: &str,
        body: Option<&T>,
    ) -> Result<JsValue, JsValue> {
        let url = format!("{}{}", self.base_url, path);
        let headers = Headers::new().map_err(|_| "Failed to create headers")?;
        headers
            .set("Content-Type", "application/json")
            .map_err(|_| "Failed to set header")?;
        if let Some(token) = &self.token {
            let auth = format!("Bearer {}", token);
            headers
                .set("Authorization", &auth)
                .map_err(|_| "Failed to set auth header")?;
        }
        let init = RequestInit::new();
        init.set_method(method);
        init.set_headers(&headers);
        if let Some(b) = body {
            let body_str = serde_json::to_string(b).map_err(|e| e.to_string())?;
            let body_js = JsValue::from_str(&body_str);
            init.set_body(&body_js);
        }
        let request =
            Request::new_with_str_and_init(&url, &init).map_err(|_| "Failed to create request")?;
        let window = window().ok_or("No window")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP error {}", resp.status())));
        }
        let json = JsFuture::from(resp.json().map_err(|_| "Failed to get JSON")?).await?;
        Ok(json)
    }

    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(), JsValue> {
        let body =
            serde_json::json!({ "username": username, "email": email, "password": password });
        let json = self
            .request("POST", "/api/auth/register", Some(&body))
            .await?;
        let auth_resp: AuthResponse =
            serde_wasm_bindgen::from_value(json).map_err(|e| e.to_string())?;
        self.save_token(auth_resp.token, auth_resp.user.username, auth_resp.user.id);
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn login(&mut self, username: String, password: String) -> Result<(), JsValue> {
        let body = serde_json::json!({ "username": username, "password": password });
        let json = self.request("POST", "/api/auth/login", Some(&body)).await?;
        let auth_resp: AuthResponse =
            serde_wasm_bindgen::from_value(json).map_err(|e| e.to_string())?;
        self.save_token(auth_resp.token, auth_resp.user.username, auth_resp.user.id);
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn create_post(&self, title: String, content: String) -> Result<(), JsValue> {
        let body = serde_json::json!({ "title": title, "content": content });
        self.request("POST", "/api/posts", Some(&body)).await?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn load_posts(&self) -> Result<JsValue, JsValue> {
        let json = self
            .request(
                "GET",
                "/api/posts?limit=100&offset=0",
                None as Option<&serde_json::Value>,
            )
            .await?;
        #[derive(Deserialize)]
        struct ListResp {
            posts: Vec<Post>,
        }
        let list: ListResp = serde_wasm_bindgen::from_value(json).map_err(|e| e.to_string())?;
        to_value(&list.posts).map_err(|e| e.to_string().into())
    }

    #[wasm_bindgen]
    pub async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<(), JsValue> {
        let body = serde_json::json!({ "title": title, "content": content });
        self.request("PUT", &format!("/api/posts/{}", id), Some(&body))
            .await?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn delete_post(&self, id: i64) -> Result<(), JsValue> {
        self.request(
            "DELETE",
            &format!("/api/posts/{}", id),
            None as Option<&serde_json::Value>,
        )
        .await?;
        Ok(())
    }
}
