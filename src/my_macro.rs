#[macro_export]
#[cfg(feature = "dp_all")]
macro_rules! debug_print {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[macro_export]
#[cfg(not(feature = "dp_all"))]
macro_rules! debug_print {
    ($( $args:expr ),*) => {}
}

#[macro_export]
#[cfg(feature = "dp_api_server")]
macro_rules! dp_api_server {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[macro_export]
#[cfg(not(feature = "dp_api_server"))]
macro_rules! dp_api_server {
    ($( $args:expr ),*) => {}
}

#[macro_export]
#[cfg(feature = "dp_scheduler")]
macro_rules! dp_scheduler {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[macro_export]
#[cfg(not(feature = "dp_scheduler"))]
macro_rules! dp_scheduler {
    ($( $args:expr ),*) => {}
}

#[macro_export]
#[cfg(feature = "dp_kubelet")]
macro_rules! dp_kubelet {
    ($( $args:expr ),*) => { println!( $( $args ),* ); };
}

#[macro_export]
#[cfg(not(feature = "dp_kubelet"))]
macro_rules! dp_kubelet {
    ($( $args:expr ),*) => {}
}
