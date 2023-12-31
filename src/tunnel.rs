//! Owned 6in4 and 4in6 tunnels with automatic deletion on drop.

use crate::{Error, Result};

use std::ffi::{c_char, c_int, CString};
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr};

const SIOCADDTUNNEL: c_int = 0x89F0 + 1;
const SIOCDELTUNNEL: c_int = 0x89F0 + 2;

/// A handle to a 6in4 tunnel. The interface is automatically deleted on drop.
#[derive(Debug)]
pub struct Sit {
    name: String,
}

impl Drop for Sit {
    fn drop(&mut self) {
        let _ = self.do_delete();
    }
}

impl Sit {
    /// Creates a new 6in4 tunnel on a parent device.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tunnel to be created.
    /// * `master` - The name of the parent interface for actual traffic.
    /// * `laddr` - The address of the local tunnel endpoint,
    ///             e.g. the WAN IPv4 address of a router.
    /// * `raddr` - The address of the remote tunnel endpoint, e.g. a tunnel server.
    pub fn new(name: String, master: String, laddr: Ipv4Addr, raddr: Ipv4Addr) -> Result<Self> {
        let tnlname = CString::new(&*name)?;
        let ifmaster = CString::new(&*master)?;
        let sit0 = CString::new("sit0")?;

        #[allow(clippy::unnecessary_cast)]
        let tnlname_raw = unsafe { &*(tnlname.as_bytes() as *const _ as *const [c_char]) };
        let mut tnlname_arr = [0; libc::IFNAMSIZ];
        for (&i, o) in tnlname_raw.iter().zip(tnlname_arr.iter_mut()) {
            *o = i;
        }

        #[allow(clippy::unnecessary_cast)]
        let sit0_raw = unsafe { &*(sit0.as_bytes() as *const _ as *const [c_char]) };
        let mut sit0_arr = [0; libc::IFNAMSIZ];
        for (&i, o) in sit0_raw.iter().zip(sit0_arr.iter_mut()) {
            *o = i;
        }

        let mut vihl = VerIhl::default();

        vihl.set_version(4);
        vihl.set_ihl(5);

        let p = IpTunnelParm4 {
            name: tnlname_arr,
            link: unsafe { libc::if_nametoindex(ifmaster.as_ptr()) },
            i_flags: 0,
            o_flags: 0,
            i_key: 0,
            o_key: 0,
            iph: IpHdr4 {
                vihl,
                tos: 0,
                tot_len: 0,
                id: 0,
                frag_off: 0,
                check: 0,
                ttl: 64,
                protocol: libc::IPPROTO_IPV6 as u8,
                saddr: u32::from(laddr).to_be(),
                daddr: u32::from(raddr).to_be(),
            },
        };

        if p.link == 0 {
            return Err(Error::LinkNotFound(master));
        }

        let ifr = IfReq4 {
            name: sit0_arr,
            ifru_data: &p,
        };

        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_IP) };
        if fd < 0 {
            return Err(io::Error::last_os_error().into());
        }

        if unsafe { libc::ioctl(fd, SIOCADDTUNNEL, &ifr) } < 0 {
            return Err(io::Error::last_os_error().into());
        }

        // Errors are safe to ignore because they don't affect tunnel creation
        // but do leave the program in an inconsistent state.
        unsafe {
            libc::close(fd);
        }

        Ok(Self { name })
    }

    fn do_delete(&self) -> Result<()> {
        delete_tunnel(&self.name)
    }
}

/// A handle to a 4in6 tunnel. The interface is automatically deleted on drop.
#[derive(Debug)]
pub struct IpIp6 {
    name: String,
}

impl Drop for IpIp6 {
    fn drop(&mut self) {
        let _ = self.do_delete();
    }
}

impl IpIp6 {
    /// Creates a new 4in6 tunnel on a parent device.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tunnel to be created.
    /// * `master` - The name of the parent interface for actual traffic.
    /// * `laddr` - The address of the local tunnel endpoint, e.g. the IPv6 GUA of a DS-Lite B4.
    /// * `raddr` - The address of the remote tunnel endpoint, e.g. a DS-Lite AFTR.
    pub fn new(name: String, master: String, laddr: Ipv6Addr, raddr: Ipv6Addr) -> Result<Self> {
        let tnlname = CString::new(&*name)?;
        let ifmaster = CString::new(&*master)?;
        let ip6tnl0 = CString::new("ip6tnl0")?;

        #[allow(clippy::unnecessary_cast)]
        let tnlname_raw = unsafe { &*(tnlname.as_bytes() as *const _ as *const [c_char]) };
        let mut tnlname_arr = [0; libc::IFNAMSIZ];
        for (&i, o) in tnlname_raw.iter().zip(tnlname_arr.iter_mut()) {
            *o = i;
        }

        #[allow(clippy::unnecessary_cast)]
        let ip6tnl0_raw = unsafe { &*(ip6tnl0.as_bytes() as *const _ as *const [c_char]) };
        let mut ip6tnl0_arr = [0; libc::IFNAMSIZ];
        for (&i, o) in ip6tnl0_raw.iter().zip(ip6tnl0_arr.iter_mut()) {
            *o = i;
        }

        let p = IpTunnelParm6 {
            name: tnlname_arr,
            link: unsafe { libc::if_nametoindex(ifmaster.as_ptr()) },
            i_flags: 0,
            o_flags: 0,
            i_key: 0,
            o_key: 0,
            iph: IpHdr6 {
                saddr: u128::from(laddr).to_be(),
                daddr: u128::from(raddr).to_be(),
            },
        };

        if p.link == 0 {
            return Err(Error::LinkNotFound(master));
        }

        let ifr = IfReq6 {
            name: ip6tnl0_arr,
            ifru_data: &p,
        };

        let fd = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, libc::IPPROTO_IP) };
        if fd < 0 {
            return Err(io::Error::last_os_error().into());
        }

        if unsafe { libc::ioctl(fd, SIOCADDTUNNEL, &ifr) } < 0 {
            return Err(io::Error::last_os_error().into());
        }

        // Errors are safe to ignore because they don't affect tunnel creation
        // but do leave the program in an inconsistent state.
        unsafe {
            libc::close(fd);
        }

        Ok(Self { name })
    }

    fn do_delete(&self) -> Result<()> {
        delete_tunnel(&self.name)
    }
}

fn delete_tunnel(name: &str) -> Result<()> {
    let tnlname = CString::new(name)?;

    #[allow(clippy::unnecessary_cast)]
    let tnlname_raw = unsafe { &*(tnlname.as_bytes() as *const _ as *const [c_char]) };
    let mut tnlname_arr = [0; libc::IFNAMSIZ];
    for (&i, o) in tnlname_raw.iter().zip(tnlname_arr.iter_mut()) {
        *o = i;
    }

    let p = IpTunnelParm4 {
        name: tnlname_arr,
        link: 0,
        i_flags: 0,
        o_flags: 0,
        i_key: 0,
        o_key: 0,
        iph: IpHdr4 {
            vihl: VerIhl::default(),
            tos: 0,
            tot_len: 0,
            id: 0,
            frag_off: 0,
            ttl: 0,
            protocol: 0,
            check: 0,
            saddr: 0,
            daddr: 0,
        },
    };

    let ifr = IfReq4 {
        name: tnlname_arr,
        ifru_data: &p,
    };

    let fd = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, libc::IPPROTO_IP) };
    if fd < 0 {
        return Err(io::Error::last_os_error().into());
    }

    if unsafe { libc::ioctl(fd, SIOCDELTUNNEL, &ifr) } < 0 {
        return Err(io::Error::last_os_error().into());
    }

    // Errors are safe to ignore because they don't affect tunnel deletion
    // but do leave the program in an inconsistent state.
    unsafe {
        libc::close(fd);
    }

    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct VerIhl(u8);

impl VerIhl {
    fn set_version(&mut self, version: u8) {
        self.0 = (self.0 & 0x0f) | (version << 4);
    }

    fn set_ihl(&mut self, ihl: u8) {
        self.0 = (self.0 & 0xf0) | (ihl % 0x0f);
    }
}

#[derive(Debug)]
#[repr(C)]
struct IpHdr4 {
    vihl: VerIhl,
    tos: u8,
    tot_len: u16,
    id: u16,
    frag_off: u16,
    ttl: u8,
    protocol: u8,
    check: u16,
    saddr: u32,
    daddr: u32,
}

#[derive(Debug)]
#[repr(C)]
struct IpTunnelParm4 {
    name: [c_char; libc::IFNAMSIZ],
    link: u32,
    i_flags: u16,
    o_flags: u16,
    i_key: u32,
    o_key: u32,
    iph: IpHdr4,
}

#[derive(Debug)]
#[repr(C)]
struct IfReq4 {
    name: [c_char; libc::IFNAMSIZ],
    ifru_data: *const IpTunnelParm4,
}

#[derive(Debug)]
#[repr(C)]
struct IpHdr6 {
    saddr: u128,
    daddr: u128,
}

#[derive(Debug)]
#[repr(C)]
struct IpTunnelParm6 {
    name: [c_char; libc::IFNAMSIZ],
    link: u32,
    i_flags: u16,
    o_flags: u16,
    i_key: u32,
    o_key: u32,
    iph: IpHdr6,
}

#[derive(Debug)]
#[repr(C)]
struct IfReq6 {
    name: [c_char; libc::IFNAMSIZ],
    ifru_data: *const IpTunnelParm6,
}
