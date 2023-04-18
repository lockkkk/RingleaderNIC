#![allow(non_camel_case_types, unused)]
#![feature(maybe_uninit_uninit_array, new_uninit)]
#![feature(try_blocks)]

use catnip::{
    file_table::FileDescriptor,
    interop::{
        dmtr_qresult_t,
        dmtr_qtoken_t,
        dmtr_sgarray_t,
    },
    libos::LibOS,
    logging,
    protocols::{
        ethernet2::MacAddress,
        ip,
        ipv4,
    },
    runtime::Runtime,
};
use clap::{
    App,
    Arg,
};
use std::collections::HashMap;
use libc::{
    c_char,
    c_int,
    sockaddr,
    socklen_t,
};
use std::{
    cell::RefCell,
    convert::TryFrom,
    ffi::{
        CStr,
        CString,
    },
    fs::File,
    io::Read,
    mem,
    net::Ipv4Addr,
    slice,
};
use yaml_rust::{
    Yaml,
    YamlLoader,
};

pub mod dpdk;
pub mod runtime;
pub mod memory;
pub mod db;

use crate::runtime::IxyRuntime;
use anyhow::{
    format_err,
    Error,
};

thread_local! {
    static LIBOS: RefCell<Option<LibOS<IxyRuntime>>> = RefCell::new(None);
}
fn with_libos<T>(f: impl FnOnce(&mut LibOS<IxyRuntime>) -> T) -> T {
    LIBOS.with(|l| {
        let mut tls_libos = l.borrow_mut();
        f(tls_libos.as_mut().expect("Uninitialized engine"))
    })
}

#[no_mangle]
pub extern "C" fn catnip_libos_noop() {
    println!("hey there!");
}

#[no_mangle]
pub extern "C" fn dmtr_init(argc: c_int, argv: *mut *mut c_char) -> c_int {
    // let r: Result<_, Error> = try {
    //     let config_path = match std::env::var("CONFIG_PATH") {
    //         Ok(s) => s,
    //         Err(..) => {
    //             if argc == 0 || argv.is_null() {
    //                 Err(format_err!("Arguments not provided"))?;
    //             }
    //             let argument_ptrs = unsafe { slice::from_raw_parts(argv, argc as usize) };
    //             let arguments: Vec<_> = argument_ptrs
    //                 .into_iter()
    //                 .map(|&p| unsafe { CStr::from_ptr(p).to_str().expect("Non-UTF8 argument") })
    //                 .collect();
    //             let matches = App::new("libos-catnip")
    //                 .arg(
    //                     Arg::with_name("config")
    //                     .short("c")
    //                     .long("config-path")
    //                     .value_name("FILE")
    //                     .help("YAML file for DPDK configuration")
    //                     .takes_value(true),
    //                 )
    //                 .arg(
    //                     Arg::with_name("iterations")
    //                     .short("i")
    //                     .long("iterations")
    //                     .value_name("COUNT")
    //                     .help("Number of iterations")
    //                     .takes_value(true),
    //                 )
    //                 .arg(
    //                     Arg::with_name("size")
    //                     .short("s")
    //                     .long("size")
    //                     .value_name("BYTES")
    //                     .help("Packet size")
    //                     .takes_value(true),
    //                 )
    //                 .get_matches_from(&arguments);

    //             matches
    //                 .value_of("config")
    //                 .ok_or_else(|| format_err!("--config-path argument not provided"))?
    //                 .to_owned()
    //         },
    //     };

    //     let mut config_s = String::new();
    //     File::open(config_path)?.read_to_string(&mut config_s)?;
    //     let config = YamlLoader::load_from_str(&config_s)?;

    //     let config_obj = match &config[..] {
    //         &[ref c] => c,
    //         _ => Err(format_err!("Wrong number of config objects"))?,
    //     };

    //     let local_ipv4_addr: Ipv4Addr = config_obj["catnip"]["my_ipv4_addr"]
    //         .as_str()
    //         .ok_or_else(|| format_err!("Couldn't find my_ipv4_addr in config"))?
    //         .parse()?;
    //     if local_ipv4_addr.is_unspecified() || local_ipv4_addr.is_broadcast() {
    //         Err(format_err!("Invalid IPv4 address"))?;
    //     }

    //     let mut arp_table = HashMap::new();
    //     if let Some(arp_table_obj) = config_obj["catnip"]["arp_table"].as_hash() {
    //         for (k, v) in arp_table_obj {
    //             let link_addr_str = k
    //                 .as_str()
    //                 .ok_or_else(|| format_err!("Couldn't find ARP table link_addr in config"))?;
    //             let link_addr = MacAddress::parse_str(link_addr_str)?;
    //             let ipv4_addr: Ipv4Addr = v
    //                 .as_str()
    //                 .ok_or_else(|| format_err!("Couldn't find ARP table link_addr in config"))?
    //                 .parse()?;
    //             arp_table.insert(ipv4_addr, link_addr);
    //         }
    //         println!("Pre-populating ARP table: {:?}", arp_table);
    //     }

    //     let mut disable_arp = false;
    //     if let Some(arp_disabled) = config_obj["catnip"]["disable_arp"].as_bool() {
    //         disable_arp = arp_disabled;
    //         println!("ARP disabled: {:?}", disable_arp);
    //     }

    //     let eal_init_args = match config_obj["dpdk"]["eal_init"] {
    //         Yaml::Array(ref arr) => arr
    //             .iter()
    //             .map(|a| {
    //                 a.as_str()
    //                     .ok_or_else(|| format_err!("Non string argument"))
    //                     .and_then(|s| CString::new(s).map_err(|e| e.into()))
    //             })
    //         .collect::<Result<Vec<_>, Error>>()?,
    //         _ => Err(format_err!("Malformed YAML config"))?,
    //     };

    //     let use_jumbo_frames = true;
    //     let mtu = 9216;
    //     let mss = 9000;
    //     let tcp_checksum_offload = true;
    //     let udp_checksum_offload = true;
    //     // let runtime = self::dpdk::initialize_ixy(
    //     //     local_ipv4_addr,
    //     //     &eal_init_args,
    //     //     arp_table,
    //     //     disable_arp,
    //     //     use_jumbo_frames,
    //     //     mtu,
    //     //     mss,
    //     //     tcp_checksum_offload,
    //     //     udp_checksum_offload,
    //     // )?;
    //     // logging::initialize();
    //     // LibOS::new(runtime)?
    // };
    // let libos = match r {
    //     Ok(libos) => libos,
    //     Err(e) => {
    //         eprintln!("Initialization failure: {:?}", e);
    //         return libc::EINVAL;
    //     },
    // };

    // LIBOS.with(move |l| {
    //     let mut tls_libos = l.borrow_mut();
    //     assert!(tls_libos.is_none());
    //     *tls_libos = Some(libos);
    // });

    0
}

#[no_mangle]
pub extern "C" fn dmtr_socket(
    qd_out: *mut c_int,
    domain: c_int,
    socket_type: c_int,
    protocol: c_int,
) -> c_int {
    with_libos(|libos| match libos.socket(domain, socket_type, protocol) {
        Ok(fd) => {
            unsafe { *qd_out = fd as c_int };
            0
        },
        Err(e) => {
            eprintln!("dmtr_socket failed: {:?}", e);
            e.errno()
        },
    })
}

#[no_mangle]
pub extern "C" fn dmtr_bind(qd: c_int, saddr: *const sockaddr, size: socklen_t) -> c_int {
    if saddr.is_null() {
        return libc::EINVAL;
    }
    if size as usize != mem::size_of::<libc::sockaddr_in>() {
        return libc::EINVAL;
    }
    let saddr_in = unsafe { *mem::transmute::<*const sockaddr, *const libc::sockaddr_in>(saddr) };
    let mut addr = Ipv4Addr::from(u32::from_be_bytes(saddr_in.sin_addr.s_addr.to_le_bytes()));
    let port = ip::Port::try_from(u16::from_be(saddr_in.sin_port)).unwrap();

    with_libos(|libos| {
        if addr.is_unspecified() {
            addr = libos.rt().local_ipv4_addr();
        }
        let endpoint = ipv4::Endpoint::new(addr, port);
        match libos.bind(qd as FileDescriptor, endpoint) {
            Ok(..) => 0,
            Err(e) => {
                eprintln!("dmtr_bind failed: {:?}", e);
                e.errno()
            },
        }
    })
}

#[no_mangle]
pub extern "C" fn dmtr_listen(fd: c_int, backlog: c_int) -> c_int {
    with_libos(
        |libos| match libos.listen(fd as FileDescriptor, backlog as usize) {
            Ok(..) => 0,
            Err(e) => {
                eprintln!("listen failed: {:?}", e);
                e.errno()
            },
        },
    )
}

#[no_mangle]
pub extern "C" fn dmtr_accept(qtok_out: *mut dmtr_qtoken_t, sockqd: c_int) -> c_int {
    with_libos(|libos| {
        unsafe { *qtok_out = libos.accept(sockqd as FileDescriptor).unwrap() };
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_connect(
    qtok_out: *mut dmtr_qtoken_t,
    qd: c_int,
    saddr: *const sockaddr,
    size: socklen_t,
) -> c_int {
    if saddr.is_null() {
        return libc::EINVAL;
    }
    if size as usize != mem::size_of::<libc::sockaddr_in>() {
        return libc::EINVAL;
    }
    let saddr_in = unsafe { *mem::transmute::<*const sockaddr, *const libc::sockaddr_in>(saddr) };
    let addr = Ipv4Addr::from(u32::from_be_bytes(saddr_in.sin_addr.s_addr.to_le_bytes()));
    let port = ip::Port::try_from(u16::from_be(saddr_in.sin_port)).unwrap();
    let endpoint = ipv4::Endpoint::new(addr, port);

    with_libos(|libos| {
        unsafe { *qtok_out = libos.connect(qd as FileDescriptor, endpoint).unwrap() };
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_close(qd: c_int) -> c_int {
    with_libos(|libos| match libos.close(qd as FileDescriptor) {
        Ok(..) => 0,
        Err(e) => {
            eprintln!("dmtr_close failed: {:?}", e);
            e.errno()
        },
    })
}

#[no_mangle]
pub extern "C" fn dmtr_push(
    qtok_out: *mut dmtr_qtoken_t,
    qd: c_int,
    sga: *const dmtr_sgarray_t,
) -> c_int {
    if sga.is_null() {
        return libc::EINVAL;
    }
    let sga = unsafe { &*sga };
    with_libos(|libos| {
        unsafe { *qtok_out = libos.push(qd as FileDescriptor, sga).unwrap() };
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_pushto(
    qtok_out: *mut dmtr_qtoken_t,
    qd: c_int,
    sga: *const dmtr_sgarray_t,
    saddr: *const sockaddr,
    size: socklen_t,
) -> c_int {
    if sga.is_null() {
        return libc::EINVAL;
    }
    let sga = unsafe { &*sga };
    if saddr.is_null() {
        return libc::EINVAL;
    }
    if size as usize != mem::size_of::<libc::sockaddr_in>() {
        return libc::EINVAL;
    }
    let saddr_in = unsafe { *mem::transmute::<*const sockaddr, *const libc::sockaddr_in>(saddr) };
    let addr = Ipv4Addr::from(u32::from_be_bytes(saddr_in.sin_addr.s_addr.to_le_bytes()));
    let port = ip::Port::try_from(u16::from_be(saddr_in.sin_port)).unwrap();
    let endpoint = ipv4::Endpoint::new(addr, port);
    with_libos(|libos| {
        unsafe { *qtok_out = libos.pushto(qd as FileDescriptor, sga, endpoint).unwrap() };
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_pop(qtok_out: *mut dmtr_qtoken_t, qd: c_int) -> c_int {
    with_libos(|libos| {
        unsafe { *qtok_out = libos.pop(qd as FileDescriptor).unwrap() };
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_poll(qr_out: *mut dmtr_qresult_t, qt: dmtr_qtoken_t) -> c_int {
    with_libos(|libos| match libos.poll(qt) {
        None => libc::EAGAIN,
        Some(r) => {
            unsafe { *qr_out = r };
            0
        },
    })
}

#[no_mangle]
pub extern "C" fn dmtr_drop(qt: dmtr_qtoken_t) -> c_int {
    with_libos(|libos| {
        libos.drop_qtoken(qt);
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_wait(qr_out: *mut dmtr_qresult_t, qt: dmtr_qtoken_t) -> c_int {
    with_libos(|libos| {
        let (qd, r) = libos.wait2(qt);
        if !qr_out.is_null() {
            let packed = dmtr_qresult_t::pack(libos.rt(), r, qd, qt);
            unsafe { *qr_out = packed };
        }
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_wait_any(
    qr_out: *mut dmtr_qresult_t,
    ready_offset: *mut c_int,
    qts: *mut dmtr_qtoken_t,
    num_qts: c_int,
) -> c_int {
    let qts = unsafe { slice::from_raw_parts(qts, num_qts as usize) };
    with_libos(|libos| {
        let (ix, qr) = libos.wait_any(qts);
        unsafe {
            *qr_out = qr;
            *ready_offset = ix as c_int;
        }
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_sgaalloc(size: libc::size_t) -> dmtr_sgarray_t {
    with_libos(|libos| {
        libos.rt().alloc_sgarray(size)
    })
}

#[no_mangle]
pub extern "C" fn dmtr_sgafree(sga: *mut dmtr_sgarray_t) -> c_int {
    if sga.is_null() {
        return 0;
    }
    with_libos(|libos| {
        libos.rt().free_sgarray(unsafe {*sga});
        0
    })
}

// #[no_mangle]
// pub extern "C" fn dmtr_queue(qd_out: *mut c_int) -> c_int {
//     unimplemented!()
// }

#[no_mangle]
pub extern "C" fn dmtr_is_qd_valid(flag_out: *mut c_int, qd: c_int) -> c_int {
    with_libos(|libos| {
        let is_valid = libos.is_qd_valid(qd as FileDescriptor);
        unsafe { *flag_out = if is_valid { 1 } else { 0 }; }
        0
    })
}

#[no_mangle]
pub extern "C" fn dmtr_open2(qd_out: *mut c_int, pathname: *const c_char, flags: c_int, mode: libc::mode_t) -> c_int {
    unimplemented!();
}
// #[no_mangle]
// pub extern "C" fn dmtr_getsockname(qd: c_int, saddr: *mut sockaddr, size: *mut socklen_t) -> c_int {
//     unimplemented!();
// }
