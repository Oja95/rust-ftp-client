use crate::client::Runtime;
use rustls::{ClientConfig, RootCertStore};
use rustls_native_certs::load_native_certs;
use rustls_pemfile::certs;
use std::fs::File;
use std::{env, io};

pub async fn configure_tls(config: &mut Runtime) {
    let mut root_store = RootCertStore::empty();

    for cert in load_native_certs().expect("Failed to load native platform certs") {
        root_store.add(cert).unwrap();
    }

    let custom_cert_path = env::var("LOCALHOST_SELF_SIGNED_CERT_PATH").ok();
    if let Some(cert_file) = custom_cert_path.map(|path| File::open(path).ok()).flatten() {
        let mut reader = io::BufReader::new(cert_file);
        for cert_result in certs(&mut reader) {
            match cert_result {
                Ok(cert) => {
                    root_store.add(cert).expect("Failed to add custom certificate!");
                }
                Err(_) => {
                    eprintln!("Warning: Failed to add custom certificate.");
                }
            }
        }
    }

    // Configure TLS with the root store
    let conf = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    config.tls_config = Some(conf);
}