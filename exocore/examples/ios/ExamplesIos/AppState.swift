
import SwiftUI
import Combine
import KeychainSwift
import Exocore

class AppState: ObservableObject {
    private let keychain = KeychainSwift()

    let objectWillChange = PassthroughSubject<AppState, Never>()

    @Published var node: LocalNode?
    @Published var nodeConfig: Exocore_Core_LocalNodeConfig?
    @Published var currentError: String?
    @Published var forceDiscovery: Bool = false

    var currentView: Page {
        get {
            if forceDiscovery || !nodeHasCell {
                return .discovery
            }

            return .list
        }
    }

    var nodeHasCell: Bool {
        get {
            (self.nodeConfig?.cells.count ?? 0) > 0
        }
    }

    static func fromPersisted() -> AppState {
        let state = AppState()

        if let configData = state.keychain.getData("node"),
           let nodeConfig = try? Exocore_Core_LocalNodeConfig(serializedData: configData),
           let node = try? LocalNode.from(config: nodeConfig) {
            state.node = node
            state.nodeConfig = nodeConfig
        }

        if state.node == nil {
            state.node = try? LocalNode.generate()
            state.refreshNodeConfig()
        }

        state.maybeInitializeExocore()
        state.triggerChanged()

        return state
    }

    func refreshNodeConfig() {
        // update latest config
        if let node = self.node,
           let newConfig = try? node.config() {
            self.nodeConfig = newConfig
        }

        // save config to keychain
        if let config = self.nodeConfig {
            let configData = try! config.serializedData()
            self.keychain.set(configData, forKey: "node")
        }

        self.maybeInitializeExocore()
        self.triggerChanged()
    }

    func maybeInitializeExocore() {
        if let node = self.node, self.nodeHasCell {
            do {
                try ExocoreClient.initialize(node: node)
                self.currentError = nil
            } catch {
                print("Error initializing client with configured node: \(error)")
                self.node = nil
                self.nodeConfig = nil
                self.currentError = error.localizedDescription
            }
        }
    }

    func triggerChanged() {
        self.objectWillChange.send(self)
    }
}

enum Page {
    case discovery
    case list
}
