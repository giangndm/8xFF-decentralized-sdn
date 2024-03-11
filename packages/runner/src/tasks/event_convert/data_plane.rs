use sans_io_runtime::{bus::BusEvent, Owner, Task, TaskInput, TaskOutput, WorkerInnerOutput};

use crate::tasks::{
    controller_plane,
    data_plane::{self, DataPlaneTask},
    SdnChannel, SdnEvent, SdnExtOut, SdnSpawnCfg,
};

///
///
/// This function will convert the input from SDN into Plane task input.
/// It only accept bus events from the SDN task.
///
pub fn convert_input<'a>(event: TaskInput<'a, SdnChannel, SdnEvent>) -> TaskInput<'a, data_plane::ChannelIn, data_plane::EventIn> {
    match event {
        TaskInput::Bus(SdnChannel::DataPlane(channel), SdnEvent::DataPlane(event)) => TaskInput::Bus(channel, event),
        TaskInput::Net(event) => TaskInput::Net(event),
        _ => panic!("Invalid input type for DataPlane {:?}", event),
    }
}

///
///
/// This function will convert the output from the Plane task into the output for the SDN task.
/// It only accept bus events from the Plane task.
///
pub fn convert_output<'a>(
    worker: u16,
    event: TaskOutput<'a, data_plane::ChannelIn, data_plane::ChannelOut, data_plane::EventOut>,
) -> WorkerInnerOutput<'a, SdnExtOut, SdnChannel, SdnEvent, SdnSpawnCfg> {
    match event {
        TaskOutput::Bus(BusEvent::ChannelSubscribe(channel)) => {
            WorkerInnerOutput::Task(Owner::group(worker, DataPlaneTask::TYPE), TaskOutput::Bus(BusEvent::ChannelSubscribe(SdnChannel::DataPlane(channel))))
        }
        TaskOutput::Bus(BusEvent::ChannelUnsubscribe(channel)) => {
            WorkerInnerOutput::Task(Owner::group(worker, DataPlaneTask::TYPE), TaskOutput::Bus(BusEvent::ChannelUnsubscribe(SdnChannel::DataPlane(channel))))
        }
        TaskOutput::Bus(BusEvent::ChannelPublish(_, safe, event)) => match event {
            data_plane::EventOut::Data(remote, data) => WorkerInnerOutput::Task(
                Owner::group(worker, DataPlaneTask::TYPE),
                TaskOutput::Bus(BusEvent::ChannelPublish(
                    SdnChannel::ControllerPlane(()),
                    safe,
                    SdnEvent::ControllerPlane(controller_plane::EventIn::Data(remote, data)),
                )),
            ),
        },
        TaskOutput::Net(out) => WorkerInnerOutput::Task(Owner::group(worker, DataPlaneTask::TYPE), TaskOutput::Net(out)),
        _ => panic!("Invalid output type from DataPlane {:?}", event),
    }
}
