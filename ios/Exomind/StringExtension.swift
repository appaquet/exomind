import Foundation

extension String {
    func nonEmpty() -> String? {
        if !self.isEmpty {
            return self
        } else {
            return nil
        }
    }
}
