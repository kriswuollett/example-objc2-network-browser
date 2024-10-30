#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use std::{
    ffi::{self, CString},
    process::ExitCode,
    ptr::{self, NonNull},
};

use block2::{Block, RcBlock};
use objc2::{
    extern_protocol,
    rc::Retained,
    runtime::{Bool, NSObjectProtocol, ProtocolObject},
    ProtocolType,
};
use tokio::signal;

extern_protocol!(
    pub unsafe trait OS_nw_browse_result: NSObjectProtocol {}
    unsafe impl ProtocolType for dyn OS_nw_browse_result {}
);
pub type nw_browser_result_t = ProtocolObject<dyn OS_nw_browse_result>;

extern_protocol!(
    pub unsafe trait OS_nw_browser: NSObjectProtocol {}
    unsafe impl ProtocolType for dyn OS_nw_browser {}
);
pub type nw_browser_t = ProtocolObject<dyn OS_nw_browser>;

extern_protocol!(
    pub unsafe trait OS_nw_endpoint: NSObjectProtocol {}
    unsafe impl ProtocolType for dyn OS_nw_endpoint {}
);
pub type nw_endpoint_t = ProtocolObject<dyn OS_nw_endpoint>;

extern_protocol!(
    pub unsafe trait OS_nw_parameters: NSObjectProtocol {}
    unsafe impl ProtocolType for dyn OS_nw_parameters {}
);
pub type nw_parameters_t = ProtocolObject<dyn OS_nw_parameters>;

extern_protocol!(
    pub unsafe trait OS_nw_browse_descriptor: NSObjectProtocol {}
    unsafe impl ProtocolType for dyn OS_nw_browse_descriptor {}
);
pub type nw_browse_descriptor_t = ProtocolObject<dyn OS_nw_browse_descriptor>;

#[link(name = "Network", kind = "framework")]
extern "C" {
    pub fn nw_browse_descriptor_create_application_service(
        application_service_name: *const ffi::c_char,
    ) -> *mut nw_browse_descriptor_t;

    pub fn nw_browse_descriptor_create_bonjour_service(
        type_: *const ffi::c_char,
        domain: *const ffi::c_char,
    ) -> *mut nw_browse_descriptor_t;

    pub fn nw_browser_create(
        descriptor: &nw_browse_descriptor_t,
        parameters: Option<&nw_parameters_t>,
    ) -> *mut nw_browser_t;

    pub fn nw_browser_set_browse_results_changed_handler(
        browser: &nw_browser_t,
        handler: Option<
            &Block<dyn Fn(NonNull<nw_browser_result_t>, NonNull<nw_browser_result_t>, Bool)>,
        >,
    );

    pub fn nw_browser_start(browser: &nw_browser_t);
}

#[tokio::main]
async fn main() -> ExitCode {
    println!("Hello, world!");

    let service_type = "_tcp._http";
    let domain: Option<&str> = None;

    unsafe {
        let service_type_ = CString::new(service_type).unwrap();
        let domain_ = domain.map(|x| CString::new(x).unwrap());

        let browse_descriptor = Retained::from_raw(nw_browse_descriptor_create_bonjour_service(
            service_type_.as_ptr(),
            domain_.map(|x| x.as_ptr()).unwrap_or(ptr::null()),
        ))
        .unwrap();

        let browser = Retained::from_raw(nw_browser_create(&browse_descriptor, None)).unwrap();

        let f = |_r1: NonNull<nw_browser_result_t>,
                 _r2: NonNull<nw_browser_result_t>,
                 changed: Bool| { println!("changed: {}", changed.as_raw()) };
        let handler = RcBlock::new(f);

        nw_browser_set_browse_results_changed_handler(&browser, Some(&handler));

        nw_browser_start(&browser);
    }

    println!("Waiting for events...");

    shutdown_signal().await;

    ExitCode::SUCCESS
}

async fn shutdown_signal() {
    let ctrl_c = async { signal::ctrl_c().await.unwrap() };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
