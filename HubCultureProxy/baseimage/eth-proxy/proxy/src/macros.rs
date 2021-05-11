
#[macro_export]
macro_rules! wrap_errs {
    ($($name:ident => $inner:ty,)*) => {

        #[derive(Debug)]
        pub enum Error {
            $(
                $name($inner),
            )*
        }

        impl ::std::fmt::Display for Error {

            fn fmt(&self, f: &mut ::std::fmt::Formatter) ->  ::std::fmt::Result {
                match self {
                    $(
                        Error::$name(err) => err.fmt(f),
                    )*
                }
            }
        }

        impl ::std::error::Error for Error {

            fn description(&self) -> &str {
                match self {
                    $(
                        Error::$name(err) => err.description(),
                    )*
                }
            }

            fn cause(&self) -> Option<&::std::error::Error> {
                match self {
                    $(
                        Error::$name(err) => Some(err),
                    )*
                }
            }
        }

        $(
            impl From<$inner> for Error {

                fn from(err: $inner) -> Self { Error::$name(err) }
            }

        )*
    }
}


#[cfg(test)]
mod tests {
    use std::{io,net};

    wrap_errs!(
        Addr => net::AddrParseError,
        Io => io::Error,
    );

    #[test]
    fn wrapper_error() {
        let parse_addr = |s: &str| -> Result<net::SocketAddr,Error> {
            let addr = s.parse()?;
            Ok(addr)
        };
        let err = parse_addr("not-an-addr").unwrap_err();
        match err {
            Error::Addr(_) => {},
            _ => panic!("Expecting `Addr` error variant")
        }
    }
}
