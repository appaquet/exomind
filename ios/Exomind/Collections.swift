import Foundation
import Exocore

class Collections {
    static let instance = Collections()
    static let PINNED_WEIGHT: UInt64 = 5000000000000

    private let queue = DispatchQueue(label: "io.exomind.collections")
    private var collectionQuery: ManagedQuery?
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

    func entityParentsPillData(entity: EntityExt, onCollectionClick: ((EntityExt) -> Void)? = nil) -> [CollectionPillData] {
        self.queue.sync {
            self.innerEntityParentsPillData(entity: entity, context: LineageContext(), onCollectionClick: onCollectionClick)
        }
    }

    fileprivate func innerEntityParentsPillData(entity: EntityExt, context: LineageContext, onCollectionClick: ((EntityExt) -> Void)? = nil) -> [CollectionPillData] {
        if context.contains(entity.id) {
            return []
        }

        let parentRelations = entity.traitsOfType(Exomind_Base_V1_CollectionChild.self)
        return parentRelations.compactMap { parentRelation in
            guard let collection = self._collections[parentRelation.message.collection.entityID] else {
                return nil
            }

            let onClick = onCollectionClick.map { inner in
                {
                    inner(collection.entity)
                }
            }
            return collection.toCollectionPill(onClick: onClick, context: context.expanded(entity.id))
        }
    }

    @objc private func onNodeReset() {
        self.maybeRunQuery()
    }

    private func maybeRunQuery() {
        if !ExocoreUtils.nodeHasCell {
            return
        }

        let query = QueryBuilder.withTrait(Exomind_Base_V1_Collection.self).count(9999).build()
        self.collectionQuery = ManagedQuery(query: query) { [weak self] in
            guard let this = self else {
                return
            }

            print("Collections > Collections have changed")
            this.indexCollections(this.collectionQuery?.results ?? [])
        }
    }

    private func indexCollections(_ entityResults: [Exocore_Store_EntityResult]) {
        self.queue.async {
            self._collections = [:]
            for entityResult in entityResults {
                let entity = entityResult.entity.toExtension()
                guard let collection = entity.traitOfType(Exomind_Base_V1_Collection.self) else {
                    print("Collections > Expected entity \(entity.id) to have a collection trait.")
                    continue
                }

                self._collections[entity.id] = CollectionEntity(entity: entity, collection: collection)
            }
            self._loaded = true

            NotificationCenter.default.post(name: .exomindCollectionsChanged, object: nil)
        }
    }

    static func hasParent(entity: EntityExt, parentId: String) -> Bool {
        let parentRel = self.getParentRelation(entity: entity, parentId: parentId)
        return parentRel != nil
    }

    static func getParentRelation(entity: EntityExt, parentId: String) -> TraitInstance<Exomind_Base_V1_CollectionChild>? {
        entity
                .traitsOfType(Exomind_Base_V1_CollectionChild.self)
                .first(where: { $0.message.collection.entityID == parentId })
    }

    static func isPinnedInParent(_ entity: EntityExt, parentId: EntityId) -> Bool {
        guard let parentRelation = Collections.getParentRelation(entity: entity, parentId: parentId) else {
            return false
        }
        return parentRelation.message.weight >= Collections.PINNED_WEIGHT
    }
}

fileprivate class CollectionEntity {
    let entity: EntityExt
    let collection: TraitInstance<Exomind_Base_V1_Collection>

    init(entity: EntityExt, collection: TraitInstance<Exomind_Base_V1_Collection>, parents: [CollectionEntity] = []) {
        self.entity = entity
        self.collection = collection
    }

    fileprivate func toCollectionPill(onClick: (() -> Void)? = nil, context: LineageContext = LineageContext()) -> CollectionPillData {
        var shortestParent: CollectionPillData?
        let parentPills = Collections.instance.innerEntityParentsPillData(entity: self.entity, context: context)
        for parentPill in parentPills {
            if shortestParent == nil {
                shortestParent = parentPill
            } else if let curShortest = shortestParent, parentPill.lineageLength() < curShortest.lineageLength() {
                shortestParent = parentPill
            }
        }

        let icon = ObjectsIcon.icon(forAnyTrait: self.collection, color: UIColor.black, dimension: CollectionPillView.ICON_SIZE)
        return CollectionPillData(id: self.entity.id, name: self.collection.strippedDisplayName, icon: icon, parent: shortestParent, onClick: onClick)
    }
}

fileprivate class LineageContext {
    var loadedIds: Set<String> = Set()

    init(_ loadedIds: Set<String> = Set()) {
        self.loadedIds = loadedIds
    }

    func expanded(_ id: String) -> LineageContext {
        var ids = Set(self.loadedIds)
        ids.insert(id)
        return LineageContext(ids)
    }

    func contains(_ id: String) -> Bool {
        self.loadedIds.contains(id)
    }
}

extension Notification.Name {
    static let exomindCollectionsChanged = Notification.Name("exomindCollectionsChanged")
}
