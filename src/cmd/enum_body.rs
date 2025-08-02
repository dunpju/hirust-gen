pub enum Enum {
    #[allow(dead_code)]
    Body { code: i32, message: &'static str },
}

impl Enum {
    #[allow(dead_code)]
    pub fn code(&self) -> i32 {
        match &self {
            Enum::Body { code, message: _ } => *code
        }
    }

    #[allow(dead_code)]
    pub fn message(&self) -> &'static str {
        match &self {
            Enum::Body { code: _, message } => *message
        }
    }
}