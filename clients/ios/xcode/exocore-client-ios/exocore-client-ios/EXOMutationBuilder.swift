import Foundation
import SwiftProtobuf

public class EXOMutationBuilder {
    private var inner: Exocore_Index_EntityMutation

    init() {
        self.inner = Exocore_Index_EntityMutation()
    }

    public static func createEntity(entityId: String? = nil) -> EXOMutationBuilder {
        let builder = EXOMutationBuilder()
        if let someEntityId = entityId {
            builder.inner.entityID = someEntityId
        } else {
            builder.inner.entityID = EXOGenerateId(prefix: "entity")
        }
        return builder
    }

    public static func updateEntity(entityId: String) -> EXOMutationBuilder {
        let builder = EXOMutationBuilder()
        builder.inner.entityID = entityId
        return builder
    }

    public func putTrait(trait: Message, traitId: String? = nil) throws -> EXOMutationBuilder {
        var putTrait = Exocore_Index_PutTraitMutation()
        putTrait.trait = Exocore_Index_Trait()
        if let someTraitId = traitId {
            putTrait.trait.id = someTraitId
        }
        putTrait.trait.creationDate = Google_Protobuf_Timestamp(date: Date())
        putTrait.trait.message = try Google_Protobuf_Any(message: trait)
        self.inner.putTrait = putTrait

        return self
    }

    public func deleteTrait(traitId: String) -> EXOMutationBuilder {
        var deleteTrait = Exocore_Index_DeleteTraitMutation()
        deleteTrait.traitID = traitId
        self.inner.deleteTrait = deleteTrait

        return self
    }

    public func build() -> Exocore_Index_EntityMutation {
        return self.inner
    }
}
