import Foundation
import SwiftProtobuf

public class MutationBuilder {
    private var entityId: String
    private var inner: Exocore_Store_MutationRequest

    init(entityId: String) {
        self.entityId = entityId
        self.inner = Exocore_Store_MutationRequest()
    }

    public static func createEntity(entityId: String? = nil) -> MutationBuilder {
        let builder = MutationBuilder(entityId: entityId ?? generateId(prefix: "entity"))
        return builder
    }

    public static func updateEntity(entityId: String) -> MutationBuilder {
        MutationBuilder(entityId: entityId)
    }

    public func putTrait(message: Message, traitId: String? = nil) throws -> MutationBuilder {
        var trait = Exocore_Store_Trait()
        trait.id = traitId ?? generateId(prefix: "trt")
        trait.creationDate = Google_Protobuf_Timestamp(date: Date())
        trait.message = try Google_Protobuf_Any(message: message)

        return try self.putTrait(trait: trait)
    }

    public func putTrait(trait: Exocore_Store_Trait) throws -> MutationBuilder {
        var putTrait = Exocore_Store_PutTraitMutation()
        putTrait.trait = trait

        var et = Exocore_Store_EntityMutation()
        et.entityID = self.entityId
        et.putTrait = putTrait
        self.inner.mutations.append(et)

        return self
    }

    public func deleteTrait(traitId: String) -> MutationBuilder {
        var deleteTrait = Exocore_Store_DeleteTraitMutation()
        deleteTrait.traitID = traitId

        var et = Exocore_Store_EntityMutation()
        et.entityID = self.entityId
        et.deleteTrait = deleteTrait
        self.inner.mutations.append(et)

        return self
    }

    public func deleteEntity() -> MutationBuilder {
        var et = Exocore_Store_EntityMutation()
        et.entityID = self.entityId
        et.deleteEntity = Exocore_Store_DeleteEntityMutation()
        self.inner.mutations.append(et)

        return self
    }

    public func returnEntities() -> MutationBuilder {
        self.inner.returnEntities = true

        return self
    }

    public func build() -> Exocore_Store_MutationRequest {
        self.inner
    }
}
