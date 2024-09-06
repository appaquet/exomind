import Foundation
import SwiftProtobuf

let traitUrlPrefix = "type.googleapis.com/"

extension Exocore_Store_Entity {
    public func trait<T: Message>(_ id: String) -> TypedTrait<T>? {
        self.traits.first(where: { (t: Exocore_Store_Trait) in
            t.id == id
        }).flatMap({ (t: Exocore_Store_Trait) in
            guard let msg = try? T.init(unpackingAny: t.message) else { return nil }
            return TypedTrait(trait: t, message: msg)
        })
    }

    public func traitOfType<T: Message>(_ message: T.Type) -> TypedTrait<T>? {
        self.traitsOfType(message).first
    }

    public func traitsOfType<T: Message>(_ message: T.Type) -> [TypedTrait<T>] {
        self.traits.compactMap({ (trait: Exocore_Store_Trait) -> TypedTrait<T>? in
            if trait.message.isA(message) {
                if let msg = try? T.init(unpackingAny: trait.message) {
                    return TypedTrait(trait: trait, message: msg)
                }
            }

            return nil
        })
    }
}

public struct TypedTrait<T: Message> {
    public let trait: Exocore_Store_Trait
    public let message: T
}

public func traitName(fromTypeUrl url: String) -> String {
    if url.hasPrefix(traitUrlPrefix) {
        return String(url.dropFirst(traitUrlPrefix.count))
    }

    return url
}

extension Exocore_Store_Trait {
    public func canonicalFullName() -> String {
        traitName(fromTypeUrl: self.message.typeURL)
    }
}

