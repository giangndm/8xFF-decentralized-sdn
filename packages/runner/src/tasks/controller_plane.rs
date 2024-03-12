use std::{collections::VecDeque, net::SocketAddr, time::Instant};

use atm0s_sdn_identity::NodeAddr;
use atm0s_sdn_network::{
    controller_plane::{ControllerPlane, Input as ControllerInput, Output as ControllerOutput, Service},
    event::DataEvent,
};
use atm0s_sdn_router::shadow::ShadowRouterDelta;
use sans_io_runtime::{bus::BusEvent, Task, TaskInput, TaskOutput};

use crate::time::{TimePivot, TimeTicker};

pub type ChannelIn = ();
pub type ChannelOut = ();

pub struct ControllerPlaneCfg {
    pub node_id: u32,
    pub tick_ms: u64,
    pub services: Vec<Box<dyn Service>>,
    #[cfg(feature = "vpn")]
    pub vpn_tun_device: sans_io_runtime::backend::tun::TunDevice,
}

#[derive(Debug, Clone)]
pub enum EventIn {
    ConnectTo(NodeAddr),
    Data(SocketAddr, DataEvent),
}

#[derive(Debug)]
pub enum EventOut {
    Data(SocketAddr, DataEvent),
    RouterRule(ShadowRouterDelta<SocketAddr>),
}

pub struct ControllerPlaneTask {
    #[allow(unused)]
    node_id: u32,
    controller: ControllerPlane,
    queue: VecDeque<TaskOutput<'static, ChannelIn, ChannelOut, EventOut>>,
    ticker: TimeTicker,
    timer: TimePivot,
    #[cfg(feature = "vpn")]
    vpn_tun_device: sans_io_runtime::backend::tun::TunDevice,
}

impl ControllerPlaneTask {
    pub fn build(cfg: ControllerPlaneCfg) -> Self {
        Self {
            node_id: cfg.node_id,
            controller: ControllerPlane::new(cfg.node_id, cfg.services),
            queue: VecDeque::from([TaskOutput::Bus(BusEvent::ChannelSubscribe(()))]),
            ticker: TimeTicker::build(1000),
            timer: TimePivot::build(),
            #[cfg(feature = "vpn")]
            vpn_tun_device: cfg.vpn_tun_device,
        }
    }

    fn map_output<'a>(output: ControllerOutput) -> TaskOutput<'a, ChannelIn, ChannelOut, EventOut> {
        match output {
            ControllerOutput::Data(remote, msg) => TaskOutput::Bus(BusEvent::ChannelPublish((), true, EventOut::Data(remote, msg))),
            ControllerOutput::RouterRule(rule) => TaskOutput::Bus(BusEvent::ChannelPublish((), true, EventOut::RouterRule(rule))),
            ControllerOutput::ShutdownSuccess => TaskOutput::Destroy,
        }
    }
}

impl Task<ChannelIn, ChannelOut, EventIn, EventOut> for ControllerPlaneTask {
    /// The type identifier for the task.
    const TYPE: u16 = 0;

    fn on_tick<'a>(&mut self, now: Instant) -> Option<TaskOutput<'a, ChannelIn, ChannelOut, EventOut>> {
        if self.ticker.tick(now) {
            self.controller.on_tick(self.timer.timestamp_ms(now));
        }
        self.pop_output(now)
    }

    fn on_event<'a>(&mut self, now: Instant, input: TaskInput<'a, ChannelIn, EventIn>) -> Option<TaskOutput<'a, ChannelIn, ChannelOut, EventOut>> {
        let now_ms = self.timer.timestamp_ms(now);
        match input {
            TaskInput::Bus(_, EventIn::Data(remote, data)) => {
                self.controller.on_event(now_ms, ControllerInput::Data(remote, data));
            }
            TaskInput::Bus(_, EventIn::ConnectTo(addr)) => {
                self.controller.on_event(now_ms, ControllerInput::ConnectTo(addr));
            }
            _ => {}
        };
        self.pop_output(now)
    }

    fn pop_output<'a>(&mut self, now: Instant) -> Option<TaskOutput<'a, ChannelIn, ChannelOut, EventOut>> {
        let now_ms = self.timer.timestamp_ms(now);
        if let Some(output) = self.controller.pop_output(now_ms) {
            self.queue.push_back(Self::map_output(output));
        }
        self.queue.pop_front()
    }

    fn shutdown<'a>(&mut self, now: Instant) -> Option<TaskOutput<'a, ChannelIn, ChannelOut, EventOut>> {
        self.controller.on_event(self.timer.timestamp_ms(now), ControllerInput::ShutdownRequest);
        self.pop_output(now)
    }
}
