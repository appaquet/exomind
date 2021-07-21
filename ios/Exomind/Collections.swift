import Foundation
import Exocore

class Collections {
    static let instance = Collections()

    private var collectionQuery: QueryStreamHandle?
    private var _loaded = false
    private var _collections: [String: CollectionEntity] = [:]

    var loaded: Bool {
        get {
            self._loaded
        }
    }

    init() {
        NotificationCenter.default.addObserver(self, selector: #selector(onNodeReset), name: .exocoreNodeReset, object: nil)
        self.maybeRunQuery()
    }

    func entityParentsPillData(entity: EntityExt, context: CollectionLineageContext = CollectionLineageContext(), onCollectionClick: ((EntityExt) -> Void)? = nil) -> [CollectionPillData] {
        let parentRelations = entity.traitsOfType(Exomind_Base_CollectionChild.self)

        return parentRelations.compactMap { parentRelation in
            guard let collection = self._collections[parentRelation.message.collection.entityID] else {
                return nil
            }

            return collection.toCollectionPill(onClick: {
                onCollectionClick?(collection.entity)
            }, context: context)
        }
    }

    @objc private func onNodeReset() {
        self.maybeRunQuery()
    }

    private func maybeRunQuery() {
        if !ExocoreUtils.nodeHasCell {
            return
        }

        let query = QueryBuilder.withTrait(Exomind_Base_Collection.self).count(9999).build()
        self.collectionQuery = ExocoreClient.store.watchedQuery(query: query) { [weak self] (status, results) in
            guard let this = self else {
                return
            }

            if (status != .running) {
                // TODO: restart
                return
            }

            print("Collections > Collections have changed")
            this.indexCollections(results?.entities ?? [])
        }
    }

    private func indexCollections(_ entityResults: [Exocore_Store_EntityResult]) {
        self._collections = [:]
        for entityResult in entityResults {
            let entity = entityResult.entity.toExtension()
            guard let collection = entity.traitOfType(Exomind_Base_Collection.self) else {
                print("Collections > Expected entity \(entity.id) to have a collection trait.")
                continue
            }

            self._collections[entity.id] = CollectionEntity(entity: entity, collection: collection)
        }
        self._loaded = true

        NotificationCenter.default.post(name: .exomindCollectionsChanged, object: nil)
    }
}

class CollectionEntity {
    let entity: EntityExt
    let collection: TraitInstance<Exomind_Base_Collection>

    init(entity: EntityExt, collection: TraitInstance<Exomind_Base_Collection>, parents: [CollectionEntity] = []) {
        self.entity = entity
        self.collection = collection
    }

    func toCollectionPill(onClick: (() -> Void)? = nil, context: CollectionLineageContext = CollectionLineageContext()) -> CollectionPillData {
        var shortestParent: CollectionPillData?
        if !context.contains(self.entity.id) {
            let parentPills = Collections.instance.entityParentsPillData(entity: self.entity, context: context.expanded(self.entity.id))
            for parentPill in parentPills {
                if shortestParent == nil {
                    shortestParent = parentPill
                } else if let curShortest = shortestParent, parentPill.lineageLength() < curShortest.lineageLength() {
                    shortestParent = parentPill
                }
            }
        }

        if shortestParent?.id == "favorites" {
            shortestParent = nil
        }

        let icon = ObjectsIcon.icon(forAnyTrait: self.collection, color: UIColor.black, dimension: CollectionPillView.ICON_SIZE)
        return CollectionPillData(id: self.entity.id, name: self.collection.strippedDisplayName, icon: icon, parent: shortestParent, onClick: onClick)
    }
}

class CollectionLineageContext {
    var loadedIds: Set<String> = Set()

    init(_ loadedIds: Set<String> = Set()) {
        self.loadedIds = loadedIds
    }

    func expanded(_ id: String) -> CollectionLineageContext {
        var ids = Set(self.loadedIds)
        ids.insert(id)
        return CollectionLineageContext(ids)
    }

    func contains(_ id: String) -> Bool {
        self.loadedIds.contains(id)
    }
}

extension Notification.Name {
    static let exomindCollectionsChanged = Notification.Name("exomindCollectionsChanged")
}
