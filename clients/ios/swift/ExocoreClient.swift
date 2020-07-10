import UIKit

public class ExocoreClient {
    public static var defaultInstance: ClientInstance?

    @discardableResult
    public static func initialize(config: Exocore_Core_LocalNodeConfig, defaultInstance: Bool = true) throws -> ClientInstance {
        let instance = try ClientInstance(config: config)
        if (defaultInstance) {
            ExocoreClient.defaultInstance = instance
        }
        return instance
    }

    @discardableResult
    public static func initialize(yamlConfig: String, defaultInstance: Bool = true) throws -> ClientInstance {
        let instance = try ClientInstance(yamlConfig: yamlConfig)
        if (defaultInstance) {
            ExocoreClient.defaultInstance = instance
        }
        return instance
    }

    static func contextFromConfig(configData: Data, format: UInt8) throws -> OpaquePointer {
        try configData.withUnsafeBytes { (ptr) -> OpaquePointer in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress
            let res = exocore_context_new(addr, UInt(configData.count), format);

            if res.status == UInt8(ExocoreQueryStatus_Success.rawValue) {
                return res.context
            } else {
                throw ExocoreError.initialization
            }
        }
    }

    public static var store: Store {
        get {
            ExocoreClient.defaultInstance!.store
        }
    }
}

public class ClientInstance {
    var context: OpaquePointer?

    public init(config: Exocore_Core_LocalNodeConfig) throws {
        let configData = try config.serializedData()

        self.context = try ExocoreClient.contextFromConfig(configData: configData, format: UInt8(ExocoreConfigFormat_Protobuf.rawValue))
    }

    public init(yamlConfig: String) throws {
        guard let configData = yamlConfig.data(using: .utf8) else {
            print("ExocoreClient > Couldn't get data from YAML string")
            throw ExocoreError.initialization
        }

        self.context = try ExocoreClient.contextFromConfig(configData: configData, format: UInt8(ExocoreConfigFormat_Yaml.rawValue))
    }

    public lazy var store: Store = {
        Store(client: self)
    }()

    deinit {
        if self.context != nil {
            // free context, which will trigger all query to fail and get freed
            exocore_context_free(self.context)
        }
    }
}

public enum ConfigFormat {
    case protobuf
    case yaml
}

public enum ExocoreError: Error {
    case initialization
}

public func GenerateId(prefix: String? = nil) -> String {
    let idPtr = exocore_generate_id(prefix)
    let idStr = String(cString: idPtr!)
    exocore_free_string(idPtr)
    return idStr
}
