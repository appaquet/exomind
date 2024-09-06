import Foundation
import Exocore

class Commands {
    static func addToParent(entity: EntityExt, parentId: String, weight: UInt64? = nil) {
        Commands.addToParent(entities: [entity], parentId: parentId, weight: weight)
    }

    static func addToParent(entities: [EntityExt], parentId: String, weight: UInt64? = nil) {
        for entity in entities {
            var mutation = MutationBuilder.updateEntity(entityId: entity.id)
            try! addChildMutation(parentId: parentId, builder: &mutation, weight: weight)
            ExocoreClient.store.mutate(mutation: mutation.build())
        }
    }

    static func removeFromParent(entity: EntityExt, parentId: String) {
        Commands.removeFromParent(entities: [entity], parentId: parentId)
    }

    static func removeFromParent(entities: [EntityExt], parentId: String) {
        for entity in entities {
            guard let parentRel = Collections.getParentRelation(entity: entity, parentId: parentId) else {
                continue
            }

            let mutation = MutationBuilder
                    .updateEntity(entityId: entity.id)
                    .deleteTrait(traitId: parentRel.id)
            ExocoreClient.store.mutate(mutation: mutation.build())
        }
    }

    static func snooze(entity: EntityExt, date: Date) {
        Commands.snooze(entities: [entity], date: date)
    }

    static func snooze(entities: [EntityExt], date: Date) {
        for entity in entities {
            var snoozed = Exomind_Base_V1_Snoozed()
            snoozed.untilDate = date.toProtobuf()

            let mutation = try! MutationBuilder
                    .updateEntity(entityId: entity.id)
                    .putTrait(message: snoozed, traitId: "snoozed")
                    .build()

            ExocoreClient.store.mutate(mutation: mutation)
        }
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
        addToParent(entities: [entity], parentId: parentId, weight: weight)
    }

    static func unpinEntityInParent(entity: EntityExt, parentId: String) {
        addToParent(entities: [entity], parentId: parentId)
    }

    static func delete(_ entity: EntityExt) {
        let mutation = MutationBuilder
                .updateEntity(entityId: entity.id)
                .deleteEntity()
                .build()

        ExocoreClient.store.mutate(mutation: mutation)
    }

    static func createTask(_ parentId: EntityId?, callback: ((EntityCreateResult) -> Void)? = nil) {
        do {
            var task = Exomind_Base_V1_Task()
            task.title = "New task"

            var builder = try MutationBuilder
                    .createEntity()
                    .returnEntities()
                    .putTrait(message: task)

            try Commands.addChildMutation(parentId: parentId ?? "inbox", builder: &builder)
            Commands.executeNewEntityMutation(mutation: builder.build(), callback: callback)
        } catch {
            print("Error creating task: \(error)")
            callback?(.failed(error))
        }
    }

    static func createNote(_ parentId: EntityId?, callback: ((EntityCreateResult) -> Void)? = nil) {
        do {
            var note = Exomind_Base_V1_Note()
            note.title = "New note"

            var builder = try MutationBuilder
                    .createEntity()
                    .returnEntities()
                    .putTrait(message: note)

            try Commands.addChildMutation(parentId: parentId ?? "inbox", builder: &builder)
            Commands.executeNewEntityMutation(mutation: builder.build(), callback: callback)
        } catch {
            print("Error creating note: \(error)")
            callback?(.failed(error))
        }
    }

    static func createEmail(_ parentId: EntityId?, callback: ((EntityCreateResult) -> Void)? = nil) {
        do {
            let email = Exomind_Base_V1_DraftEmail()

            var builder = try MutationBuilder
                    .createEntity()
                    .returnEntities()
                    .putTrait(message: email)

            try Commands.addChildMutation(parentId: parentId ?? "inbox", builder: &builder)
            Commands.executeNewEntityMutation(mutation: builder.build(), callback: callback)
        } catch {
            print("Error creating email: \(error)")
            callback?(.failed(error))
        }
    }

    static func createCollection(_ parentId: EntityId?, callback: ((EntityCreateResult) -> Void)? = nil) {
        do {
            var collection = Exomind_Base_V1_Collection()
            collection.name = "New collection"

            var builder = try MutationBuilder
                    .createEntity()
                    .returnEntities()
                    .putTrait(message: collection)

            try Commands.addChildMutation(parentId: parentId ?? "inbox", builder: &builder)
            Commands.executeNewEntityMutation(mutation: builder.build(), callback: callback)
        } catch {
            print("Error creating collection: \(error)")
            callback?(.failed(error))
        }
    }

    static func addChildMutation(parentId: EntityId, builder: inout MutationBuilder, weight: UInt64? = nil) throws {
        var child = Exomind_Base_V1_CollectionChild()
        child.collection.entityID = parentId
        child.weight = weight ?? UInt64(Date().millisecondsSince1970)

        builder = try builder.putTrait(message: child, traitId: "child_\(parentId)")
    }

    static func executeNewEntityMutation(mutation: Exocore_Store_MutationRequest, callback: ((EntityCreateResult) -> ())? = nil) {
        ExocoreClient.store.mutate(mutation: mutation, onCompletion: { (status, results) in
            DispatchQueue.main.async {
                guard let results = results,
                      results.entities.count > 0 else {
                    callback?(.success(nil))
                    return
                }

                callback?(.success(results.entities[0].toExtension()))
            }
        })
    }
}

enum EntityCreateResult {
    case success(EntityExt?)
    case failed(Error)
}