import unittest
import uuid

from grapl_common.envelope import proto_uuid_to_pyuuid, pyuuid_to_proto_uuid


class TestProtoUuidv4(unittest.TestCase):
    # A basic test to ensure that conversions between uuid representations preserve the uuid
    def test_uuid_roundtrip(self) -> None:
        original_uuid: uuid.UUID = uuid.uuid4()
        proto_uuid = pyuuid_to_proto_uuid(original_uuid)
        converted_original = proto_uuid_to_pyuuid(proto_uuid)
        converted_proto = pyuuid_to_proto_uuid(converted_original)
        self.assertEqual(original_uuid, converted_original)
        self.assertEqual(proto_uuid, converted_proto)
