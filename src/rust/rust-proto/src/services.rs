use uuid::Uuid;

pub use crate::graplinc::grapl::api::services::v1beta1::*;

pub trait ServiceMessage {
    const TYPE_NAME: &'static str;
}

// impl<M> TryFrom<(Metadata, M)> for Envelope
// where
//     M: ServiceMessage + prost::Message,
// {
//     type Error = prost::EncodeError;
//
//     fn try_from((metadata, inner_message): (Metadata, M)) -> Result<Self, Self::Error> {
//         let mut inner_message_buf = Vec::new();
//         inner_message.encode(&mut inner_message_buf)?;
//         Ok(Self {
//             metadata: Some(metadata),
//             inner_type: M::TYPE_NAME.to_string(),
//             inner_message: inner_message_buf,
//         })
//     }
// }

impl ProtoUuidV4 {
    pub fn new() -> Self {
        Self::from(uuid::Uuid::new_v4())
    }
}

impl From<ProtoUuidV4> for uuid::Uuid {
    fn from(p: ProtoUuidV4) -> Self {
        let lsb: [u8; 8] = p.lsb.to_le_bytes();
        let msb: [u8; 8] = p.msb.to_le_bytes();
        let u: u128 = u128::from_le_bytes([
            lsb[0], lsb[1], lsb[2], lsb[3], lsb[4], lsb[5], lsb[6], lsb[7], msb[0], msb[1], msb[2],
            msb[3], msb[4], msb[5], msb[6], msb[7],
        ]);

        uuid::Builder::from_u128(u).build()
    }
}

impl From<uuid::Uuid> for ProtoUuidV4 {
    fn from(u: Uuid) -> Self {
        let u = u.as_u128();
        let u_le: [u8; 16] = u.to_le_bytes();
        // Ugly, but `Index` isn't const, so Rust can't figure
        // out that half of a [u8; 16] is two [u8; 8]'s
        let lsb: u64 = u64::from_le_bytes([
            u_le[0], u_le[1], u_le[2], u_le[3], u_le[4], u_le[5], u_le[6], u_le[7],
        ]);
        let msb: u64 = u64::from_le_bytes([
            u_le[8], u_le[9], u_le[10], u_le[11], u_le[12], u_le[13], u_le[14], u_le[15],
        ]);
        Self { lsb, msb }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equality_from_uuid() {
        for _ in 0..1000 {
            // start with a Uuid
            let u0 = uuid::Uuid::new_v4();
            // Convert it into a ProtoUuidV4
            let pu: ProtoUuidV4 = u0.clone().into();
            // Then back to a Uuid
            let u1: Uuid = pu.into();
            // Hopefully it hasn't changed
            assert_eq!(u0, u1);
        }
    }

    #[test]
    fn test_equality_from_proto_uuid() {
        for _ in 0..1000 {
            // start with a ProtoUuid
            let pu0 = ProtoUuidV4::new();
            // Convert it into a Uuid
            let u: Uuid = pu0.clone().into();
            // Then back to a ProtoUuid
            let pu1: ProtoUuidV4 = u.into();
            // Hopefully it hasn't changed
            assert_eq!(pu0, pu1);
        }
    }
}
