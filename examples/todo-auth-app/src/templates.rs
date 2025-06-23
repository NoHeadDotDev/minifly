use askama::Template;
use crate::models::{SessionUser, Todo, UserApp};

#[derive(Template)]
#[template(path = "base.html")]
pub struct Base {
    pub title: &'static str,
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomePage {
    pub user: Option<SessionUser>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginPage {
    pub error: Option<String>,
    pub email: String,
}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignupPage {
    pub error: Option<String>,
    pub email: String,
    pub regions: &'static [(&'static str, &'static str)],
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardPage {
    pub user: SessionUser,
    pub todos: Vec<Todo>,
    pub regions: Vec<(UserApp, String)>,
    pub selected_region: Option<String>,
}