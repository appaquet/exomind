
import Foundation
import SwiftProtobuf

extension Message {
    func expensiveClone() -> Self? {
        do {
            let serialized = try self.serializedData()
            return try Self.init(serializedData: serialized)
        } catch {
            return nil
        }
    }
}
