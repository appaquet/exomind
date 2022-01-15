import Foundation
import Exocore

// TODO: pinEntityInParent
// TODO: unpinEntityInParent
// TODO: removeSnooze
// TODO: delete
// TODO: createNote, Task, Collection, Link

class Commands {
    static func addToParent(entity: EntityExt, parentId: String) throws {
        var mutation = MutationBuilder.updateEntity(entityId: entity.id)
        try addChildMutation(parentId: parentId, builder: &mutation)
        ExocoreClient.store.mutate(mutation: mutation.build())
    }

    static func removeFromParent(entity: EntityExt, parentId: String) {
        guard let parentRel = getEntityParentRelation(entity: entity, parentId: parentId) else {
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

    static func addChildMutation(parentId: EntityId, builder: inout MutationBuilder) throws {
        var child = Exomind_Base_V1_CollectionChild()
        child.collection.entityID = parentId
        child.weight = UInt64(Date().millisecondsSince1970)

        builder = try builder.putTrait(message: child, traitId: "child_\(parentId)")
    }

    static func hasParent(entity: EntityExt, parentId: String) -> Bool {
        let parentRel = self.getEntityParentRelation(entity: entity, parentId: parentId)
        return parentRel != nil
    }

    static func getEntityParentRelation(entity: EntityExt, parentId: String) -> TraitInstance<Exomind_Base_V1_CollectionChild>? {
        entity
                .traitsOfType(Exomind_Base_V1_CollectionChild.self)
                .first(where: { $0.message.collection.entityID == parentId })
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
