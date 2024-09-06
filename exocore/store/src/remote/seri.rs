use exocore_core::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_protos::{
    generated::{
        exocore_store::{EntityQuery, EntityResults},
        store_transport_capnp::{
            mutation_request, mutation_response, query_request, query_response,
            watched_query_request,
        },
    },
    prost::Message,
    store::{MutationRequest, MutationResult},
};

use crate::error::Error;

pub fn query_to_request_frame(
    query: &EntityQuery,
) -> Result<CapnpFrameBuilder<query_request::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<query_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    let buf = query.encode_to_vec();
    msg_builder.set_request(&buf);

    Ok(frame_builder)
}

#[cfg(feature = "local")]
pub fn query_from_request_frame<I>(
    frame: TypedCapnpFrame<I, query_request::Owned>,
) -> Result<EntityQuery, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    let data = reader.get_request()?;

    let query = EntityQuery::decode(data)?;

    Ok(query)
}

pub fn watched_query_to_request_frame(
    query: &EntityQuery,
) -> Result<CapnpFrameBuilder<watched_query_request::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<watched_query_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    let buf = query.encode_to_vec();
    msg_builder.set_request(&buf);

    Ok(frame_builder)
}

#[cfg(feature = "local")]
pub fn query_results_to_response_frame(
    result: Result<EntityResults, Error>,
) -> Result<CapnpFrameBuilder<query_response::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<query_response::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    match result {
        Ok(res) => {
            let buf = res.encode_to_vec();
            msg_builder.set_response(&buf);
        }
        Err(err) => {
            msg_builder.set_error(err.to_string().as_str());
        }
    }

    Ok(frame_builder)
}

pub fn query_results_from_response_frame<I>(
    frame: TypedCapnpFrame<I, query_response::Owned>,
) -> Result<EntityResults, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    if reader.has_error() {
        Err(Error::Remote(reader.get_error()?.to_string().map_err(
            |err| anyhow!("couldn't convert error to utf8: {err}"),
        )?))
    } else {
        let data = reader.get_response()?;
        let res = EntityResults::decode(data)?;
        Ok(res)
    }
}

pub fn mutation_to_request_frame(
    request: MutationRequest,
) -> Result<CapnpFrameBuilder<mutation_request::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<mutation_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    let buf = request.encode_to_vec();
    msg_builder.set_request(&buf);

    Ok(frame_builder)
}

#[cfg(feature = "local")]
pub fn mutation_from_request_frame<I>(
    frame: TypedCapnpFrame<I, mutation_request::Owned>,
) -> Result<MutationRequest, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    let data = reader.get_request()?;
    Ok(MutationRequest::decode(data)?)
}

#[cfg(feature = "local")]
pub fn mutation_result_to_response_frame(
    result: Result<MutationResult, Error>,
) -> Result<CapnpFrameBuilder<mutation_response::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<mutation_response::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    match result {
        Ok(res) => {
            let buf = res.encode_to_vec();
            msg_builder.set_response(&buf);
        }
        Err(err) => {
            msg_builder.set_error(err.to_string().as_str());
        }
    }

    Ok(frame_builder)
}

pub fn mutation_result_from_response_frame<I>(
    frame: TypedCapnpFrame<I, mutation_response::Owned>,
) -> Result<MutationResult, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    if reader.has_error() {
        Err(Error::Remote(reader.get_error()?.to_string().map_err(
            |err| anyhow!("couldn't convert error to utf8: {err}"),
        )?))
    } else {
        let data = reader.get_response()?;
        Ok(MutationResult::decode(data)?)
    }
}
