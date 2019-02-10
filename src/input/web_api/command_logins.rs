pub struct CompileTimeLogin {
  pub username: &'static str,
	pub password: &'static str,
}

pub const LIST: [CompileTimeLogin; 2] = [
	CompileTimeLogin {username: "eva", password: "superbabje"},
	CompileTimeLogin {username: "david", password: "12481632"},
];
