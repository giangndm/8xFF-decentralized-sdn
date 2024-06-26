use std::{collections::VecDeque, sync::Arc, time::Instant};

use atm0s_sdn_identity::NodeId;
use atm0s_sdn_network::{
    base::ServiceBuilder,
    controller_plane::{ControllerPlane, Input as ControllerInput, Output as ControllerOutput},
    features::{FeaturesControl, FeaturesEvent},
    ExtIn, ExtOut, LogicControl, LogicEvent,
};
use rand::rngs::ThreadRng;
use sans_io_runtime::{bus::BusEvent, Task, TaskInput, TaskOutput};

use crate::time::{TimePivot, TimeTicker};

pub type ChannelIn = ();
pub type ChannelOut = ();

pub struct ControllerPlaneCfg<SC, SE, TC, TW> {
    pub node_id: NodeId,
    pub session: u64,
    pub tick_ms: u64,
    pub services: Vec<Arc<dyn ServiceBuilder<FeaturesControl, FeaturesEvent, SC, SE, TC, TW>>>,
    #[cfg(feature = "vpn")]
    pub vpn_tun_device: Option<sans_io_runtime::backend::tun::TunDevice>,
}

pub type EventIn<TC> = LogicControl<TC>;
pub type EventOut<TW> = LogicEvent<TW>;

pub struct ControllerPlaneTask<SC, SE, TC, TW> {
    #[allow(unused)]
    node_id: NodeId,
    controller: ControllerPlane<SC, SE, TC, TW>,
    queue: VecDeque<TaskOutput<'static, ExtOut<SE>, ChannelIn, ChannelOut, EventOut<TW>>>,
    ticker: TimeTicker,
    timer: TimePivot,
    #[cfg(feature = "vpn")]
    _vpn_tun_device: Option<sans_io_runtime::backend::tun::TunDevice>,
}

impl<SC, SE, TC, TW> ControllerPlaneTask<SC, SE, TC, TW> {
    pub fn build(cfg: ControllerPlaneCfg<SC, SE, TC, TW>) -> Self {
        Self {
            node_id: cfg.node_id,
            controller: ControllerPlane::new(cfg.node_id, cfg.session, cfg.services, Box::new(ThreadRng::default())),
            queue: VecDeque::from([TaskOutput::Bus(BusEvent::ChannelSubscribe(()))]),
            ticker: TimeTicker::build(1000),
            timer: TimePivot::build(),
            #[cfg(feature = "vpn")]
            _vpn_tun_device: cfg.vpn_tun_device,
        }
    }
}

impl<SC, SE, TC, TW> Task<ExtIn<SC>, ExtOut<SE>, ChannelIn, ChannelOut, EventIn<TC>, EventOut<TW>> for ControllerPlaneTask<SC, SE, TC, TW> {
    /// The type identifier for the task.
    const TYPE: u16 = 0;

    fn on_tick<'a>(&mut self, now: Instant) -> Option<TaskOutput<'a, ExtOut<SE>, ChannelIn, ChannelOut, EventOut<TW>>> {
        if self.ticker.tick(now) {
            self.controller.on_tick(self.timer.timestamp_ms(now));
        }
        self.pop_output(now)
    }

    fn on_event<'a>(&mut self, now: Instant, input: TaskInput<'a, ExtIn<SC>, ChannelIn, EventIn<TC>>) -> Option<TaskOutput<'a, ExtOut<SE>, ChannelIn, ChannelOut, EventOut<TW>>> {
        let now_ms = self.timer.timestamp_ms(now);
        match input {
            TaskInput::Bus(_, event) => {
                self.controller.on_event(now_ms, ControllerInput::Control(event));
            }
            TaskInput::Ext(event) => {
                self.controller.on_event(now_ms, ControllerInput::Ext(event));
            }
            _ => {
                panic!("Invalid input type for ControllerPlane")
            }
        };
        self.pop_output(now)
    }

    fn pop_output<'a>(&mut self, now: Instant) -> Option<TaskOutput<'a, ExtOut<SE>, ChannelIn, ChannelOut, EventOut<TW>>> {
        let now_ms = self.timer.timestamp_ms(now);
        if let Some(output) = self.queue.pop_front() {
            return Some(output);
        }
        let output = self.controller.pop_output(now_ms)?;
        match output {
            ControllerOutput::Ext(event) => Some(TaskOutput::Ext(event)),
            ControllerOutput::ShutdownSuccess => Some(TaskOutput::Destroy),
            ControllerOutput::Event(bus) => Some(TaskOutput::Bus(BusEvent::ChannelPublish((), true, bus))),
        }
    }

    fn shutdown<'a>(&mut self, now: Instant) -> Option<TaskOutput<'a, ExtOut<SE>, ChannelIn, ChannelOut, EventOut<TW>>> {
        self.controller.on_event(self.timer.timestamp_ms(now), ControllerInput::ShutdownRequest);
        self.pop_output(now)
    }
}
