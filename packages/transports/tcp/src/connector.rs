use crate::connection::{TcpConnectionReceiver, TcpConnectionSender};
use crate::handshake::{outgoing_handshake, OutgoingHandshakeError};
use crate::msg::TcpMsg;
use crate::TCP_PROTOCOL_ID;
use async_bincode::futures::AsyncBincodeStream;
use async_std::channel::Sender;
use async_std::net::{Shutdown, TcpStream};
use p_8xff_sdn_identity::{ConnId, NodeAddr, NodeAddrBuilder, NodeId, Protocol};
use p_8xff_sdn_network::transport::{AsyncConnectionAcceptor, ConnectionRejectReason, OutgoingConnectionError, TransportConnector, TransportEvent, TransportOutgoingLocalUuid};
use p_8xff_sdn_utils::error_handle::ErrorUtils;
use p_8xff_sdn_utils::Timer;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct TcpConnector {
    pub(crate) seed: AtomicU64,
    pub(crate) node_id: NodeId,
    pub(crate) node_addr_builder: Arc<NodeAddrBuilder>,
    pub(crate) internal_tx: Sender<TransportEvent>,
    pub(crate) timer: Arc<dyn Timer>,
}

impl TcpConnector {
    /// Extracts a `SocketAddr` from a given `Multiaddr`.
    ///
    /// Fails if the given `Multiaddr` does not begin with an IP
    /// protocol encapsulating a TCP port.
    fn multiaddr_to_socketaddr(mut addr: NodeAddr) -> Result<SocketAddr, ()> {
        // "Pop" the IP address and TCP port from the end of the address,
        // ignoring a `/p2p/...` suffix as well as any prefix of possibly
        // outer protocols, if present.
        let mut port = None;
        while let Some(proto) = addr.pop() {
            match proto {
                Protocol::Ip4(ipv4) => match port {
                    Some(port) => return Ok(SocketAddr::new(ipv4.into(), port)),
                    None => return Err(()),
                },
                Protocol::Ip6(ipv6) => match port {
                    Some(port) => return Ok(SocketAddr::new(ipv6.into(), port)),
                    None => return Err(()),
                },
                Protocol::Tcp(portnum) => match port {
                    Some(_) => return Err(()),
                    None => port = Some(portnum),
                },
                Protocol::P2p(_) => {}
                _ => return Err(()),
            }
        }
        Err(())
    }
}

async fn wait_accept(local_uuid: TransportOutgoingLocalUuid, remote_node: NodeId, conn_id: ConnId, internal_tx: &Sender<TransportEvent>) -> Result<(), OutgoingHandshakeError> {
    log::info!("[TcpConnector] connect to {} send local check", remote_node);
    let (connection_acceptor, recv) = AsyncConnectionAcceptor::new();
    internal_tx
        .send(TransportEvent::OutgoingRequest(remote_node, conn_id, connection_acceptor, local_uuid))
        .await
        .map_err(|_| OutgoingHandshakeError::InternalError)?;
    log::info!("[TcpConnector] connect to {} wait local accept", remote_node);
    if let Err(e) = recv.recv().await.map_err(|_| OutgoingHandshakeError::InternalError)? {
        log::error!("Connection rejected {:?}", e);
        return Err(OutgoingHandshakeError::Rejected);
    }
    Ok(())
}

impl TransportConnector for TcpConnector {
    fn connect_to(&self, local_uuid: TransportOutgoingLocalUuid, remote_node_id: NodeId, remote_node_addr: NodeAddr) -> Result<(), OutgoingConnectionError> {
        log::info!("[TcpConnector] connect to node {}", remote_node_addr);
        let timer = self.timer.clone();
        let node_id = self.node_id;
        let node_addr = self.node_addr_builder.addr();
        let remote_addr = Self::multiaddr_to_socketaddr(remote_node_addr.clone()).map_err(|_| OutgoingConnectionError::UnsupportedProtocol)?;
        let conn_seed = self.seed.fetch_add(1, Ordering::Relaxed);
        let conn_id = ConnId::from_out(TCP_PROTOCOL_ID, conn_seed);
        let internal_tx = self.internal_tx.clone();
        async_std::task::spawn(async move {
            if let Err(e) = wait_accept(local_uuid, remote_node_id, conn_id, &internal_tx).await {
                log::error!("Outgoing handshake error {:?}", e);
                internal_tx
                    .send(TransportEvent::OutgoingError {
                        local_uuid,
                        node_id: remote_node_id,
                        conn_id: Some(conn_id),
                        err: OutgoingConnectionError::BehaviorRejected(ConnectionRejectReason::Custom("LocalReject".to_string())),
                    })
                    .await
                    .print_error("Should send Outgoing Error");
                return;
            }

            match TcpStream::connect(remote_addr).await {
                Ok(socket) => {
                    let mut socket_read = AsyncBincodeStream::<_, TcpMsg, TcpMsg, _>::from(socket.clone()).for_async();
                    let socket_write = AsyncBincodeStream::<_, TcpMsg, TcpMsg, _>::from(socket.clone()).for_async();
                    match outgoing_handshake(remote_node_id, node_id, node_addr, &mut socket_read, conn_id, &internal_tx).await {
                        Ok(_) => {
                            let (connection_sender, reliable_sender) = TcpConnectionSender::new(node_id, remote_node_id, remote_node_addr.clone(), conn_id, 1000, socket_write, timer.clone());
                            let connection_receiver = Box::new(TcpConnectionReceiver {
                                node_id,
                                remote_node_id,
                                remote_addr: remote_node_addr,
                                conn_id,
                                socket: socket_read,
                                timer,
                                reliable_sender,
                            });
                            internal_tx
                                .send(TransportEvent::Outgoing(Arc::new(connection_sender), connection_receiver, local_uuid))
                                .await
                                .print_error("Should send Outgoing");
                        }
                        Err(err) => {
                            socket.shutdown(Shutdown::Both).print_error("Should shutdown socket");
                            internal_tx
                                .send(TransportEvent::OutgoingError {
                                    local_uuid,
                                    node_id: remote_node_id,
                                    conn_id: Some(conn_id),
                                    err: match err {
                                        OutgoingHandshakeError::SocketError => OutgoingConnectionError::DestinationNotFound,
                                        OutgoingHandshakeError::Timeout => OutgoingConnectionError::AuthenticationError,
                                        OutgoingHandshakeError::WrongMsg => OutgoingConnectionError::AuthenticationError,
                                        OutgoingHandshakeError::InternalError => OutgoingConnectionError::AuthenticationError,
                                        OutgoingHandshakeError::Rejected => OutgoingConnectionError::AuthenticationError,
                                    },
                                })
                                .await
                                .print_error("Should send OutgoingError");
                        }
                    }
                }
                Err(err) => {
                    log::error!("TcpStream connect error {}", err);
                    internal_tx
                        .send(TransportEvent::OutgoingError {
                            local_uuid,
                            node_id: remote_node_id,
                            conn_id: Some(conn_id),
                            err: OutgoingConnectionError::DestinationNotFound,
                        })
                        .await
                        .print_error("Should send OutgoingError::DestinationNotFound");
                }
            }
        });
        Ok(())
    }
}
