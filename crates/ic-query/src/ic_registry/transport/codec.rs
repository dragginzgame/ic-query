use crate::ic_registry::RegistryFetchError;
use prost::Message;

pub(in crate::ic_registry) fn decode_message<T>(
    message: &'static str,
    bytes: &[u8],
) -> Result<T, RegistryFetchError>
where
    T: Message + Default,
{
    T::decode(bytes).map_err(|err| RegistryFetchError::ProtobufDecode {
        message,
        reason: err.to_string(),
    })
}
