
import Foundation
import SwiftProtobuf

public class MutationBuilder {
    private var entityId: String;
    private var inner: Exocore_Index_MutationRequest

    init(entityId: String) {
        self.entityId = entityId
        self.inner = Exocore_Index_MutationRequest()
    }

    public static func createEntity(entityId: String? = nil) -> MutationBuilder {
        var finalEntityId: String
        if let someEntityId = entityId {
            finalEntityId = someEntityId
        } else {
            finalEntityId = GenerateId(prefix: "entity")
        }

        let builder = MutationBuilder(entityId: finalEntityId)
        return builder
    }

    public static func updateEntity(entityId: String) -> MutationBuilder {
        MutationBuilder(entityId: entityId)
    }

    public func putTrait(trait: Message, traitId: String? = nil) throws -> MutationBuilder {
        var putTrait = Exocore_Index_PutTraitMutation()
        putTrait.trait = Exocore_Index_Trait()
        if let someTraitId = traitId {
            putTrait.trait.id = someTraitId
        }
        putTrait.trait.creationDate = Google_Protobuf_Timestamp(date: Date())
        putTrait.trait.message = try Google_Protobuf_Any(message: trait)

        var et = Exocore_Index_EntityMutation()
        et.putTrait = putTrait
        self.inner.mutations.append(et)

        return self
    }

    public func deleteTrait(traitId: String) -> MutationBuilder {
        var deleteTrait = Exocore_Index_DeleteTraitMutation()
        deleteTrait.traitID = traitId

        var et = Exocore_Index_EntityMutation()
        et.deleteTrait = deleteTrait
        self.inner.mutations.append(et)

        return self
    }

    public func deleteEntity() -> MutationBuilder {
        var et = Exocore_Index_EntityMutation()
        et.deleteEntity = Exocore_Index_DeleteEntityMutation()
        self.inner.mutations.append(et)

        return self
    }

    public func build() -> Exocore_Index_MutationRequest {
        self.inner
    }
}
