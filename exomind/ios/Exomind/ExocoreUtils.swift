
import Foundation
import Exocore
import KeychainSwift

class ExocoreUtils {
    static var node: LocalNode? = nil

    static var logFile: String = {
        let folder = try! FileManager.default.url(for: .cachesDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
        let logFile = folder.path + "/log.txt"

        return logFile
    }()

    static func initialize() throws {
        Exocore.initialize(logLevel: LogLevel.info, logFile: logFile)
        try bootNode()
    }

    static func bootNode() throws {
        if self.node == nil {
            let keyChain = KeychainSwift()
            if let configData = keyChain.getData("node"),
               let nodeConfig = try? Exocore_Core_LocalNodeConfig(serializedData: configData),
               let node = try? LocalNode.from(config: nodeConfig) {
                self.node = node
            }
        }

        if self.node == nil {
            self.node = try LocalNode.generate()
            try self.saveNode(node: self.node!)
        }

        if let node = self.node, self.nodeHasCell {
            try ExocoreClient.initialize(node: node)
            NotificationCenter.default.post(name: .exocoreNodeReset, object: nil)
        } else {
            print("ExocoreUtils > No node or cell configured")
        }
    }

    static func saveNode(node: LocalNode) throws {
        let config = try node.config()
        let configData = try config.serializedData()
        let keychain = KeychainSwift()
        keychain.set(configData, forKey: "node")
        self.node = node
    }

    static func resetTransport() {
        ExocoreClient.defaultInstance?.resetTransport()
    }

    static var nodeHasCell: Bool {
        get {
            if let node = self.node,
               let config = try? node.config() {
                return !config.cells.isEmpty
            }

            return false
        }
    }
}

extension Notification.Name {
    static let exocoreNodeReset = Notification.Name("exocoreNodeReset")
}
