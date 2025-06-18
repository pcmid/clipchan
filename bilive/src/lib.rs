mod appkey;
pub mod bapi;
mod client;
pub mod live;
pub mod session;
pub mod user;
pub mod wbi;

#[cfg(test)]
mod tests {
    use ctor::ctor;

    #[ctor]
    fn init_tracing() {
        tracing_subscriber::fmt().with_env_filter("trace").init();
    }
}
