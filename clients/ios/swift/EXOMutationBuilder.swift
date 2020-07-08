
import Foundation
import SwiftProtobuf

public class EXOMutationBuilder {
    private var entityId: String;
    private var inner: Exocore_Index_MutationRequest

    init(entityId: String) {
        self.entityId = entityId
        self.inner = Exocore_Index_MutationRequest()
    }

    public static func createEntity(entityId: String? = nil) -> EXOMutationBuilder {
        var finalEntityId: String
        if let someEntityId = entityId {
            finalEntityId = someEntityId
        } else {
            finalEntityId = EXOGenerateId(prefix: "entity")
        }

        let builder = EXOMutationBuilder(entityId: finalEntityId)
        return builder
    }

    public static func updateEntity(entityId: String) -> EXOMutationBuilder {
        EXOMutationBuilder(entityId: entityId)
    }

    public func putTrait(trait: Message, traitId: String? = nil) throws -> EXOMutationBuilder {
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

    public func deleteTrait(traitId: String) -> EXOMutationBuilder {
        var deleteTrait = Exocore_Index_DeleteTraitMutation()
        deleteTrait.traitID = traitId

        var et = Exocore_Index_EntityMutation()
        et.deleteTrait = deleteTrait
        self.inner.mutations.append(et)

        return self
    }

    public func build() -> Exocore_Index_MutationRequest {
        self.inner
    }
}
