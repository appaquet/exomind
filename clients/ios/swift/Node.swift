
import Foundation

public class LocalNode {
    var ptr: OpaquePointer

    init(ptr: OpaquePointer) {
        self.ptr = ptr
    }

    public static func generate() throws -> LocalNode {
        let res = exocore_local_node_generate()
        if res.status == ExocoreLocalNodeStatus_Error.rawValue {
            throw LocalNodeError.initialization
        }

        return LocalNode(ptr: res.node)
    }

    public static func from(config: Exocore_Core_LocalNodeConfig) throws -> LocalNode {
        let configData = try config.serializedData()
        return try configData.withUnsafeBytes { (dataPtr) throws in
            let configAddr = dataPtr.bindMemory(to: UInt8.self).baseAddress

            let res = exocore_local_node_new(configAddr, UInt(configData.count))
            if res.status == ExocoreLocalNodeStatus_Error.rawValue {
                throw LocalNodeError.initialization
            }

            return LocalNode(ptr: res.node)
        }
    }

    public func config() throws -> Exocore_Core_LocalNodeConfig {
        let bytes = exocore_local_node_protobuf_config(ptr)
        defer {
            exocore_bytes_free(bytes)
        }

        let nodeData = Data(bytes: bytes.bytes!, count: Int(bytes.size))
        return try Exocore_Core_LocalNodeConfig(serializedData: nodeData)
    }

    deinit {
        exocore_local_node_free(self.ptr)
    }
}

public enum LocalNodeError: Error {
    case initialization
}
