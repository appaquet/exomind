import Foundation
import Exocore

// TODO: createNote, Task, Collection, Link

class Commands {
    static func addToParent(entity: EntityExt, parentId: String, weight: UInt64? = nil) {
        var mutation = MutationBuilder.updateEntity(entityId: entity.id)
        try! addChildMutation(parentId: parentId, builder: &mutation, weight: weight)
        ExocoreClient.store.mutate(mutation: mutation.build())
    }

    static func removeFromParent(entity: EntityExt, parentId: String) {
        guard let parentRel = Collections.getParentRelation(entity: entity, parentId: parentId) else {
            return
        }

        let mutation = MutationBuilder
                .updateEntity(entityId: entity.id)
                .deleteTrait(traitId: parentRel.id)
        ExocoreClient.store.mutate(mutation: mutation.build())
    }

    static func snooze(entity: EntityExt, date: Date) {
        var snoozed = Exomind_Base_V1_Snoozed()
        snoozed.untilDate = date.toProtobuf()

        let mutation = try! MutationBuilder
                .updateEntity(entityId: entity.id)
                .putTrait(message: snoozed, traitId: "snoozed")
                .build()

        ExocoreClient.store.mutate(mutation: mutation)
    }

    static func removeSnooze(_ entity: EntityExt) {
        guard let snoozeTrait = entity.traitsOfType(Exomind_Base_V1_Snoozed.self).first else {
            return
        }

        let mutation = MutationBuilder
                .updateEntity(entityId: entity.id)
                .deleteTrait(traitId: snoozeTrait.id)
                .build()

        ExocoreClient.store.mutate(mutation: mutation)
    }

    static func pinEntityInParent(entity: EntityExt, parentId: String) {
        let weight = UInt64(Date().millisecondsSince1970) + Collections.PINNED_WEIGHT;
        addToParent(entity: entity, parentId: parentId, weight: weight)
    }

    static func unpinEntityInParent(entity: EntityExt, parentId: String) {
        addToParent(entity: entity, parentId: parentId)
    }

    static func delete(_ entity: EntityExt) {
        let mutation = MutationBuilder
                .updateEntity(entityId: entity.id)
                .deleteEntity()
                .build()

        ExocoreClient.store.mutate(mutation: mutation)
    }

    static func addChildMutation(parentId: EntityId, builder: inout MutationBuilder, weight: UInt64? = nil) throws {
        var child = Exomind_Base_V1_CollectionChild()
        child.collection.entityID = parentId
        child.weight = weight ?? UInt64(Date().millisecondsSince1970)

        builder = try builder.putTrait(message: child, traitId: "child_\(parentId)")
    }

    static func executeNewEntityMutation(mutation: Exocore_Store_MutationRequest, callback: ((EntityExt?) -> ())?) {
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
