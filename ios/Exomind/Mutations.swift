import Foundation
import Exocore

class Mutations {
    static func hasParent(entity: EntityExt, parentId: String) -> Bool {
        let parentRel = entity
                .traitsOfType(Exomind_Base_CollectionChild.self)
                .first(where: { $0.message.collection.entityID == parentId })

        return parentRel != nil
    }

    static func getParent(entity: EntityExt, parentId: String) -> TraitInstance<Exomind_Base_CollectionChild>? {
        entity
                .traitsOfType(Exomind_Base_CollectionChild.self)
                .first(where: { $0.message.collection.entityID == parentId })
    }

    static func addParent(entity: EntityExt, parentId: String) throws {
        var mutation = MutationBuilder.updateEntity(entityId: entity.id)
        try addChildMutation(parentId: parentId, builder: &mutation)
        ExocoreClient.store.mutate(mutation: mutation.build())
    }

    static func removeParent(entity: EntityExt, parentId: String) {
        guard let parentRel = getParent(entity: entity, parentId: parentId) else {
            return
        }

        let mutation = MutationBuilder
                .updateEntity(entityId: entity.id)
                .deleteTrait(traitId: parentRel.id)

        ExocoreClient.store.mutate(mutation: mutation.build())
    }

    static func snooze(entity: EntityExt, date: Date, callback: (() -> Void)? = nil) {
        var postpone = Exomind_Base_Postponed()
        postpone.untilDate = date.toProtobuf()

        let mutation = try! MutationBuilder
                .updateEntity(entityId: entity.id)
                .putTrait(message: postpone, traitId: "postponed")
                .returnEntities()
                .build()

        ExocoreClient.store.mutate(mutation: mutation, onCompletion: { (status, results) in
            callback?()
        })
    }

    static func addChildMutation(parentId: EntityId, builder: inout MutationBuilder) throws {
        var child = Exomind_Base_CollectionChild()
        child.collection.entityID = parentId
        child.weight = UInt64(Date().millisecondsSince1970)

        builder = try builder.putTrait(message: child, traitId: "child_\(parentId)")
    }

    static func executeCreateEntityMutation(mutation: Exocore_Index_MutationRequest, callback: ((EntityExt?) -> ())?) {
        ExocoreClient.store.mutate(mutation: mutation, onCompletion: { (status, results) in
            DispatchQueue.main.async {
                guard let results = results,
                      results.entities.count > 0 else {
                    callback?(nil)
                    return
                }

                callback?(results.entities[0].toExtension())
            }
        })
    }
}
