
import SwiftUI
import Combine
import KeychainSwift
import Exocore

class AppState: ObservableObject {
    private let keychain = KeychainSwift()

    let objectWillChange = PassthroughSubject<AppState, Never>()

    var currentView: Page = .bootstrap {
        didSet {
            self.objectWillChange.send(self)
        }
    }

    @Published var config: String?
    @Published var configError: String?

    static func fromPersisted() -> AppState {
        let state = AppState()

        state.config = state.keychain.get("config")
        state.configureExocore()

        if state.config != nil {
            state.currentView = .list
        }

        return state
    }

    func saveConfig() {
        self.configureExocore()

        if let config = self.config {
            self.keychain.set(config, forKey: "config")
            self.currentView = .list
            self.objectWillChange.send(self)
        }
    }

    func configureExocore() {
        if let config = self.config {
            do {
                try ExocoreClient.initialize(yamlConfig: config)
                self.configError = nil
            } catch {
                print("Error loading client with given config: \(error)")
                self.config = nil
                self.configError = error.localizedDescription
            }

            self.objectWillChange.send(self)
        }
    }
}

enum Page {
    case bootstrap
    case list
}
