
import Foundation

extension Array {
    func element(at index: Int) -> Element? {
        index >= 0 && index < count ? self[index] : nil
    }
}
