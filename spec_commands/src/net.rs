//! Network requests and the sending thereof
//!
//! This does not actually run any external programs yet!
//! It is just a placeholder.

use std::collections::HashMap;
use std::io::Error as IoError;
use std::net::{TcpStream, UdpSocket};
use std::sync::Mutex;
use std::time::Duration;

use log::*;

use spec_checks::tcp::{RunTcp, Request as TcpRequest};
use spec_checks::udp::{RunUdp, Request as UdpRequest};
use spec_exec::Command;

use super::GlobalOptions;


/// The **net non-command** makes network requests and caches the results.
#[derive(Debug)]
pub struct NetNonCommand {
    tcps: HashMap<TcpRequest, Mutex<Option<Option<bool>>>>,
    udps: HashMap<UdpRequest, Mutex<Option<Option<bool>>>>,
}

impl NetNonCommand {

    /// Creates a new non-command.
    pub fn create(_global_options: &impl GlobalOptions) -> Self {
        Self {
            tcps: HashMap::new(),
            udps: HashMap::new(),
        }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        std::iter::empty()
    }
}

impl RunTcp for NetNonCommand {
    fn prime(&mut self, request: &TcpRequest) {
        if ! self.tcps.contains_key(request) {
            debug!("Priming network with TCP request {:?}", request);
            self.tcps.insert(request.clone(), Mutex::new(None));
        }
    }

    fn send_tcp_request(&self, request: &TcpRequest) -> bool {
        let mut slot = self.tcps.get(request).unwrap().lock().unwrap();
        let response = slot.get_or_insert_with(|| {

            // Because TCP handshakes, we can use that to determine a successful
            // connection.
            match TcpStream::connect(request.addr()) {
                Ok(stream) => {
                    debug!("Received response -> {:?}", stream.peer_addr());
                    Some(true)
                }
                Err(e) => {
                    debug!("Network error -> {:?}", e);
                    Some(false)
                }
            }
        });

        response.clone().unwrap()
    }
}

impl RunUdp for NetNonCommand {
    fn prime(&mut self, request: &UdpRequest) {
        if ! self.udps.contains_key(request) {
            debug!("Priming network with UDP request {:?}", request);
            self.udps.insert(request.clone(), Mutex::new(None));
        }
    }

    fn send_udp_request(&self, request: &UdpRequest) -> bool {
        let mut slot = self.udps.get(request).unwrap().lock().unwrap();
        let response = slot.get_or_insert_with(|| {
            let result = test_udp(request.addr(), Duration::new(2, 0));

            if let Err(e) = &result {
                warn!("Error running network check: {:?}", e);
            }

            Some(result.is_ok())
        });
        response.clone().unwrap()

    }
}

fn test_udp(addr: (&str, u16), timeout: Duration) -> Result<(), IoError> {
    let socket = UdpSocket::bind((addr.0, 49129))?;
    socket.set_read_timeout(Some(timeout))?;
    socket.set_write_timeout(Some(timeout))?;
    socket.connect(addr)?;
    socket.send(&[0, 1, 2, 3, 4, 5])?;

    let mut buf = [0; 16];
    let received = socket.recv(&mut buf)?;
    debug!("Received {} bytes {:?}", received, &buf[..received]);
    Ok(())
}
