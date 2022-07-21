#[macro_export]
macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            None => return,
            Some(value) => value
        }
    }
}

#[macro_export]
macro_rules! unwrap_or_empty_string {
    ( $e:expr ) => {
        match $e {
            None => String::from(""),
            Some(value) => value
        }
    }
}