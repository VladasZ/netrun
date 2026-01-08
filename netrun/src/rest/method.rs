use std::fmt::{Display, Formatter};

#[derive(Copy, Clone)]
pub enum Method {
    Get,
    Post,
}

impl Method {
    pub fn get(&self) -> bool {
        matches!(self, Self::Get)
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let st = match self {
            Self::Get => "GET",
            Self::Post => "POST",
        };

        write!(f, "{st}")
    }
}
