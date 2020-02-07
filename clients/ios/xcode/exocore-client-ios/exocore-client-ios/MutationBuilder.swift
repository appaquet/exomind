
import Foundation
import SwiftProtobuf

public class MutationBuilder {
    private var inner: Exocore_Index_EntityMutation

    init() {
        self.inner = Exocore_Index_EntityMutation()
    }

    public static func createEntity(entityId: String? = nil) -> MutationBuilder {
        let builder = MutationBuilder()
        if let someEntityId = entityId {
            builder.inner.entityID = someEntityId
        } else {
            builder.inner.entityID = GenerateId(prefix: "entity")
        }
        return builder
    }

    public static func updateEntity(entityId: String) -> MutationBuilder {
        let builder = MutationBuilder()
        builder.inner.entityID = entityId
        return builder
    }

    public func putTrait(trait: Message, traitId: String? = nil) throws -> MutationBuilder {
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

    public func deleteTrait(traitId: String) -> MutationBuilder {
        var deleteTrait = Exocore_Index_DeleteTraitMutation()
        deleteTrait.traitID = traitId
        self.inner.deleteTrait = deleteTrait

        return self
    }

    public func build() -> Exocore_Index_EntityMutation {
        return self.inner
    }
}
