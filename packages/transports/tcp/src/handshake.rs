use crate::connection::{recv_tcp_stream, send_tcp_stream, BUFFER_LEN};
use crate::msg::TcpMsg;
use async_std::channel::{RecvError, Sender};
use async_std::net::TcpStream;
use bluesea_identity::{NodeAddr, NodeId};
use futures_util::io::{ReadHalf, WriteHalf};
use futures_util::AsyncWriteExt;
use network::transport::{
    AsyncConnectionAcceptor, ConnectionRejectReason, OutgoingConnectionError, TransportEvent,
};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub enum IncomingHandshakeError {
    SocketError,
    Timeout,
    WrongMsg,
    InternalError,
    Rejected,
    NetError,
    ValidateError,
}

pub async fn incoming_handshake<MSG: Serialize + DeserializeOwned>(
    my_node: NodeId,
    my_addr: NodeAddr,
    socket: &mut TcpStream,
    conn_id: u32,
    internal_tx: &Sender<TransportEvent<MSG>>,
) -> Result<(NodeId, NodeAddr), IncomingHandshakeError> {
    log::info!("[TcpTransport] handshake wait ConnectRequest");

    let mut buf = [0; BUFFER_LEN];
    let msg = async_std::future::timeout(
        Duration::from_secs(5),
        recv_tcp_stream::<MSG>(&mut buf, socket),
    )
    .await
    .map_err(|_| IncomingHandshakeError::Timeout)?
    .map_err(|_| IncomingHandshakeError::SocketError)?;
    let (remote_node, remote_addr) = match msg {
        TcpMsg::ConnectRequest(my_node_2, node, addr) => {
            if my_node_2 == my_node {
                log::info!("[TcpTransport] handshake from {} {}", node, addr);
                (node, addr)
            } else {
                log::warn!(
                    "[TcpTransport] handshake from wrong node info {} vs {}",
                    my_node_2,
                    my_node
                );
                send_tcp_stream(
                    socket,
                    TcpMsg::<MSG>::ConnectResponse(Err("WrongNode".to_string())),
                )
                .await;
                return Err(IncomingHandshakeError::ValidateError);
            }
        }
        _ => {
            log::warn!("[TcpTransport] handshake wrong msg");
            return Err(IncomingHandshakeError::WrongMsg);
        }
    };

    let (connection_acceptor, recv) = AsyncConnectionAcceptor::new();
    internal_tx
        .send(TransportEvent::IncomingRequest(
            remote_node,
            conn_id,
            connection_acceptor,
        ))
        .await
        .map_err(|_| IncomingHandshakeError::InternalError)?;
    if let Err(e) = recv
        .recv()
        .await
        .map_err(|_| IncomingHandshakeError::InternalError)?
    {
        send_tcp_stream(
            socket,
            TcpMsg::<MSG>::ConnectResponse(Err("Rejected".to_string())),
        )
        .await;
        return Err(IncomingHandshakeError::Rejected);
    }
    send_tcp_stream(
        socket,
        TcpMsg::<MSG>::ConnectResponse(Ok((my_node, my_addr))),
    )
    .await;

    Ok((remote_node, remote_addr))
}

pub enum OutgoingHandshakeError {
    SocketError,
    Timeout,
    WrongMsg,
    InternalError,
    Rejected,
    NetError,
}

pub async fn outgoing_handshake<MSG: Serialize + DeserializeOwned>(
    remote_node: NodeId,
    my_node: NodeId,
    my_node_addr: NodeAddr,
    socket: &mut TcpStream,
    conn_id: u32,
    internal_tx: &Sender<TransportEvent<MSG>>,
) -> Result<(), OutgoingHandshakeError> {
    let mut buf = [0; BUFFER_LEN];

    log::info!(
        "[TcpTransport] outgoing_handshake send ConnectRequest to {}",
        remote_node
    );
    send_tcp_stream(
        socket,
        TcpMsg::<MSG>::ConnectRequest(remote_node, my_node, my_node_addr),
    )
    .await
    .map_err(|_| OutgoingHandshakeError::SocketError)?;

    log::info!(
        "[TcpTransport] outgoing_handshake wait ConnectResponse from {}",
        remote_node
    );
    let msg = async_std::future::timeout(
        Duration::from_secs(5),
        recv_tcp_stream::<MSG>(&mut buf, socket),
    )
    .await
    .map_err(|_| {
        log::info!(
            "[TcpTransport] outgoing_handshake wait ConnectResponse from {} timeout",
            remote_node
        );
        OutgoingHandshakeError::Timeout
    })?
    .map_err(|_| OutgoingHandshakeError::SocketError)?;
    let _node = match msg {
        TcpMsg::ConnectResponse(Ok((node_id, addr))) => {
            log::info!(
                "[TcpTransport] outgoing_handshake ConnectResponse Ok from {} {}",
                node_id,
                addr
            );
            node_id
        }
        TcpMsg::ConnectResponse(Err(err)) => {
            log::info!(
                "[TcpTransport] outgoing_handshake ConnectResponse Err {}",
                err
            );
            return Err(OutgoingHandshakeError::Rejected);
        }
        _ => return Err(OutgoingHandshakeError::WrongMsg),
    };
    Ok(())
}
