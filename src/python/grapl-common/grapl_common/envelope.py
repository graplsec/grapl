from __future__ import annotations

import json
import uuid

from graplinc.grapl.api.services.v1beta1.types_pb2 import Envelope as ProtoEnvelope
from graplinc.grapl.api.services.v1beta1.types_pb2 import Metadata as ProtoMetadata
from graplinc.grapl.api.services.v1beta1.types_pb2 import ProtoUuidV4


def proto_uuid_to_pyuuid(proto_uuid: ProtoUuidV4) -> uuid.UUID:
    lsb = proto_uuid.lsb
    msb = proto_uuid.msb
    lsb_bytes = int.to_bytes(lsb, byteorder="little", length=8, signed=False)
    msb_bytes = int.to_bytes(msb, byteorder="little", length=8, signed=False)

    return uuid.UUID(bytes=lsb_bytes + msb_bytes)


def pyuuid_to_proto_uuid(pyuuid: uuid.UUID) -> ProtoUuidV4:
    lsb_bytes = pyuuid.bytes[:8]
    msb_bytes = pyuuid.bytes[8:]
    lsb = int.from_bytes(lsb_bytes, byteorder="little", signed=False)
    msb = int.from_bytes(msb_bytes, byteorder="little", signed=False)

    proto_uuid = ProtoUuidV4()
    proto_uuid.lsb = lsb
    proto_uuid.msb = msb
    return proto_uuid


class Metadata(object):
    def __init__(self, trace_id: uuid.UUID, tenant_id: uuid.UUID) -> None:
        self.trace_id = trace_id
        self.tenant_id = tenant_id

    @staticmethod
    def new() -> Metadata:
        return Metadata(trace_id=uuid.uuid4(), tenant_id=uuid.uuid4())

    @staticmethod
    def from_proto_bytes(b: bytes) -> Metadata:
        metadata = ProtoEnvelope()
        metadata.ParseFromString(b)
        return Metadata.from_proto(metadata)

    @staticmethod
    def from_proto_parts(trace_id: ProtoUuidV4, tenant_id: ProtoUuidV4) -> Metadata:
        return Metadata(proto_uuid_to_pyuuid(trace_id), proto_uuid_to_pyuuid(tenant_id))

    @staticmethod
    def from_proto(metadata: ProtoMetadata) -> Metadata:
        return Metadata.from_proto_parts(metadata.trace_id, metadata.tenant_id)

    def to_proto(self) -> ProtoMetadata:
        metadata = ProtoMetadata()
        metadata.trace_id.CopyFrom(pyuuid_to_proto_uuid(self.trace_id))
        metadata.tenant_id.CopyFrom(pyuuid_to_proto_uuid(self.tenant_id))
        return metadata

    def to_proto_bytes(self) -> bytes:
        return self.to_proto().SerializeToString()

    def __str__(self) -> str:
        return "Metadata " + json.dumps(vars(self))


class Envelope(object):
    def __init__(
        self,
        metadata: Metadata,
        inner_message: bytes,
        inner_type: str,
    ) -> None:
        self.metadata = metadata
        self.inner_message = inner_message
        self.inner_type = inner_type

    @staticmethod
    def from_proto_bytes(b: bytes) -> Envelope:
        envelope = ProtoEnvelope()
        envelope.ParseFromString(b)
        return Envelope.from_proto(envelope)

    @staticmethod
    def from_proto(envelope: ProtoEnvelope) -> Envelope:
        return Envelope.from_proto_parts(
            envelope.metadata,
            envelope.inner_message,
            envelope.inner_type,
        )

    @staticmethod
    def from_proto_parts(
        metadata: ProtoMetadata,
        inner_message: bytes,
        inner_type: str,
    ) -> Envelope:
        return Envelope(Metadata.from_proto(metadata), inner_message, inner_type)

    def to_proto(self) -> ProtoEnvelope:
        envelope = ProtoEnvelope()
        envelope.metadata.CopyFrom(self.metadata.to_proto())
        envelope.inner_message = self.inner_message
        envelope.inner_type = self.inner_type
        return envelope

    def to_proto_bytes(self) -> bytes:
        return self.to_proto().SerializeToString()

    def get_metadata(self) -> Metadata:
        return self.metadata

    def get_inner_type(self) -> str:
        return self.inner_type

    def get_inner_message(self) -> bytes:
        return self.inner_message

    def __str__(self) -> str:
        return "Envelope " + json.dumps(
            {
                "metadata": vars(self.metadata),
                "inner_type": self.inner_type,
                # The message isn't valid utf8, and it could be huge, so just print out its length
                "inner_message_len": len(self.inner_message),
            }
        )
