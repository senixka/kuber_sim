#[macro_export]
macro_rules! sim_assert {
    ($condition:expr, $msg:expr) => {
        if !($condition) {
            println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
            println!("Assertion failed: {}", $msg);
            println!("<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<");
            std::process::exit(1);
        }
    };
}

#[macro_export]
macro_rules! sim_ok {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => {
                println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
                println!("Assertion failed: {}", $msg);
                println!("<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<");
                std::process::exit(1);
            }
        }
    };
}

#[macro_export]
macro_rules! sim_some {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(val) => val,
            None => {
                println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
                println!("Assertion failed: {}", $msg);
                println!("<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<");
                std::process::exit(1);
            }
        }
    };
}

#[macro_export]
#[cfg(feature = "dp_api_server")]
macro_rules! dp_api_server {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[macro_export]
#[cfg(not(feature = "dp_api_server"))]
macro_rules! dp_api_server {
    ($( $args:expr ),*) => {};
}

#[macro_export]
#[cfg(feature = "dp_scheduler")]
macro_rules! dp_scheduler {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[macro_export]
#[cfg(not(feature = "dp_scheduler"))]
macro_rules! dp_scheduler {
    ($( $args:expr ),*) => {};
}

#[macro_export]
#[cfg(feature = "dp_kubelet")]
macro_rules! dp_kubelet {
    ($( $args:expr ),*) => { println!( $( $args ),* ); };
}

#[macro_export]
#[cfg(not(feature = "dp_kubelet"))]
macro_rules! dp_kubelet {
    ($( $args:expr ),*) => {};
}

#[macro_export]
#[cfg(feature = "dp_ca")]
macro_rules! dp_ca {
    ($( $args:expr ),*) => { println!( $( $args ),* ); };
}

#[macro_export]
#[cfg(not(feature = "dp_ca"))]
macro_rules! dp_ca {
    ($( $args:expr ),*) => {};
}

#[macro_export]
#[cfg(feature = "dp_hpa")]
macro_rules! dp_hpa {
    ($( $args:expr ),*) => { println!( $( $args ),* ); };
}

#[macro_export]
#[cfg(not(feature = "dp_hpa"))]
macro_rules! dp_hpa {
    ($( $args:expr ),*) => {};
}

#[macro_export]
#[cfg(feature = "dp_vpa")]
macro_rules! dp_vpa {
    ($( $args:expr ),*) => { println!( $( $args ),* ); };
}

#[macro_export]
#[cfg(not(feature = "dp_vpa"))]
macro_rules! dp_vpa {
    ($( $args:expr ),*) => {};
}
