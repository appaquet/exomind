
import Foundation
import Exocore
import KeychainSwift

class ExocoreUtils {
    static var initialized: Bool = false
    static var error: String?

    static var cellConfig: String? {
        get {
            let keyChain = KeychainSwift()
            return keyChain.get("cell_config")
        }
        set {
            let keyChain = KeychainSwift()
            if let newValue = newValue {
                keyChain.set(newValue, forKey: "cell_config")
            } else {
                keyChain.delete("cell_config")
            }
        }
    }

    static func initialize() {
        self.initialized = false

        do {
            try ExocoreClient.initialize(yamlConfig: cellConfig ?? "")
            self.initialized = true
        } catch {
            self.error = error.localizedDescription
        }
    }
}
