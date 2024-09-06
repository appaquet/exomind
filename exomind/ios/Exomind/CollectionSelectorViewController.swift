import UIKit
import Exocore
import SwiftUI

class CollectionSelectorViewController: UINavigationController {
    var forEntities: [EntityExt] = []
    var tableView: CollectionSelectorTableViewController!

    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView = (super.topViewController as! CollectionSelectorTableViewController)
        self.tableView.partialEntities = forEntities

        // set colors of navigation bar
        Stylesheet.styleNavigationBar(self.navigationBar, bgColor: Stylesheet.collectionSelectorNavigationBarBg, fgColor: Stylesheet.collectionSelectorNavigationBarFg)
    }
}

class CollectionSelectorTableViewController: UITableViewController, UISearchBarDelegate {
    fileprivate var partialEntities: [EntityExt] = []

    private var searchBar: UISearchBar!

    private var collectionsQuery: ManagedQuery?
    private var collectionsQueryFilter: String?

    private var entitiesQuery: QueryStreamHandle?
    private var completeEntities: [EntityExt] = []
    private var entityParentsQueries: [QueryStreamHandle] = []
    private var entityParents: [Collection]?

    private var currentFilter: String?
    private var searchDebouncer: Debouncer!
    private var collectionData: [Collection] = []

    private let bgQueue = DispatchQueue(label: "io.exomind.collection_selector")

    override func viewDidLoad() {
        super.viewDidLoad()

        self.tableView.delegate = self
        self.tableView.keyboardDismissMode = .onDrag

        self.tableView.register(SwiftUICellViewHost<CollectionSelectorCell>.self, forCellReuseIdentifier: "cell")
        self.tableView.rowHeight = UITableView.automaticDimension
        self.tableView.estimatedRowHeight = 75

        self.searchBar = UISearchBar()
        self.navigationItem.titleView = self.searchBar
        self.searchBar.becomeFirstResponder()
        self.searchBar.placeholder = "Filter"
        self.searchBar.delegate = self

        Stylesheet.styleSearchBar(self.searchBar, bgColor: Stylesheet.collectionSelectorNavigationBarBg, fgColor: Stylesheet.collectionSelectorNavigationBarFg)

        self.searchDebouncer = Debouncer(timeInterval: 0.5) { [weak self] in
            self?.loadData()
        }

        self.loadData()
    }

    private func loadData() {
        self.bgQueue.async { [weak self] in
            guard let this = self else {
                return
            }

            if this.entitiesQuery == nil {
                this.queryEntities()
            }

            if this.collectionsQuery == nil || this.collectionsQueryFilter != this.currentFilter {
                this.queryFilteredCollections()
            }

            if let collectionsResults = this.collectionsQuery?.results,
               let entityParents = this.entityParents {

                let collectionsEntities: [Collection] = collectionsResults.compactMap({ res in
                    Collection.fromEntity(entity: res.entity.toExtension())
                })
                let combined = entityParents + collectionsEntities
                var uniqueMap = [String: Collection]()
                this.collectionData = combined.filter { (col: Collection) -> Bool in
                    if uniqueMap[col.entity.id] == nil {
                        uniqueMap[col.entity.id] = col
                        return true
                    } else {
                        return false
                    }
                }

                DispatchQueue.main.async {
                    this.tableView.reloadData()
                }
            }
        }
    }

    private func queryEntities() {
        let ids = self.partialEntities.map {
            $0.id
        }
        let entityQuery = QueryBuilder.withIds(ids).build()
        self.entitiesQuery = ExocoreClient.store.watchedQuery(query: entityQuery, onChange: { [weak self] (status, res) in
            guard let this = self,
                  let res = res,
                  res.entities.count > 0 else {
                return
            }

            let entities = res.entities.map {
                $0.entity.toExtension()
            }
            this.completeEntities = entities
            this.queryEntityParents(entities)
            this.loadData()
        })
    }

    private func queryFilteredCollections() {
        let collectionsQuery: QueryBuilder
        if let currentFilter = currentFilter {
            let traitQuery = TraitQueryBuilder.matching(query: currentFilter).build()
            collectionsQuery = QueryBuilder.withTrait(Exomind_Base_V1_Collection.self, query: traitQuery).count(30)
        } else {
            collectionsQuery = QueryBuilder.withTrait(Exomind_Base_V1_Collection.self).count(30)
        }

        self.collectionsQuery = ManagedQuery(query: collectionsQuery.build(), onChange: { [weak self] in
            self?.loadData()
        })
        self.collectionsQueryFilter = currentFilter
    }

    private func queryEntityParents(_ entities: [EntityExt]) {
        self.entityParentsQueries = []
        self.entityParents = []

        for entity in entities {
            let parents = entity
                    .traitsOfType(Exomind_Base_V1_CollectionChild.self)
                    .map({ $0.message.collection.entityID })

            if !parents.isEmpty {
                let query = QueryBuilder.withIds(parents).count(100).build()
                self.entityParentsQueries.append(ExocoreClient.store.watchedQuery(query: query, onChange: { [weak self] (status, res) in
                    let parents = res?.entities.compactMap({ Collection.fromEntity(entity: $0.entity.toExtension()) }) ?? []
                    self?.entityParents?.append(contentsOf: parents)
                    self?.loadData()
                }))
            }
        }
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        // force reload data when dark/light style has changed
        if self.traitCollection.userInterfaceStyle != previousTraitCollection?.userInterfaceStyle {
            self.tableView.reloadData()
        }
    }

    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        if (!searchText.isEmpty) {
            self.currentFilter = searchText
        } else {
            self.currentFilter = nil
        }

        self.searchDebouncer.renewInterval()
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        self.collectionData.count
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: "cell", for: indexPath) as! SwiftUICellViewHost<CollectionSelectorCell>

        if let collection = self.collectionData.element(at: (indexPath as NSIndexPath).item) {
            let name = collection.trait.strippedDisplayName
            let checked = self.hasParent(parentEntityId: collection.entity.id)
            let img = ObjectsIcon.icon(forAnyTrait: collection.trait, color: UIColor.label, dimension: CollectionSelectorCell.ICON_SIZE)
            let parents = Collections.instance.entityParentsPillData(entity: collection.entity)

            let cellData = CollectionSelectorCellData(id: collection.entity.id, name: name, checked: checked, icon: img, parents: parents)
            cell.setView(view: CollectionSelectorCell(data: cellData), parentController: self)
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        self.tableView.deselectRow(at: indexPath, animated: false)

        if let collection = self.collectionData.element(at: (indexPath as NSIndexPath).item) {
            if (self.hasParent(parentEntityId: collection.entity.id)) {
                self.removeParent(parentEntityId: collection.entity.id)
            } else {
                self.addParent(parentEntityId: collection.entity.id)
            }
        }
    }

    override func scrollViewDidScroll(_ scrollView: UIScrollView) {
        let totalHeight = (scrollView.contentSize.height - tableView.frame.size.height)
        let currentPosition = scrollView.contentOffset.y
        let averageHeight = CGFloat(74)
        let itemsComingUp = (totalHeight - currentPosition) / averageHeight

        // if only 5 items or less are coming up, we load new
        let canExpand = self.collectionsQuery?.canExpand ?? false
        if (itemsComingUp < 10 && canExpand) {
            self.collectionsQuery?.expand()
        }
    }

    @IBAction func handleDoneClick(_ sender: AnyObject) {
        self.dismiss(animated: true, completion: nil)
    }

    private func hasParent(parentEntityId id: String) -> Bool {
        if self.completeEntities.isEmpty {
            return false
        }

        return self.completeEntities.allSatisfy { entity in
            Collections.hasParent(entity: entity, parentId: id)
        }
    }

    private func addParent(parentEntityId id: String) {
        if self.completeEntities.isEmpty {
            return
        }

        Commands.addToParent(entities: self.completeEntities, parentId: id)
    }

    private func removeParent(parentEntityId id: String) {
        if self.completeEntities.isEmpty {
            return
        }

        Commands.removeFromParent(entities: self.completeEntities, parentId: id)
    }

    deinit {
        print("CollectionSelectionViewController > Deinit")
    }
}

fileprivate struct Collection {
    let entity: EntityExt
    let trait: TraitInstance<Exomind_Base_V1_Collection>
    let parents: [CollectionPillData]

    static func fromEntity(entity: EntityExt) -> Collection? {
        guard let collectionTrait = entity.traitOfType(Exomind_Base_V1_Collection.self) else {
            return nil
        }
        let parents = Collections.instance.entityParentsPillData(entity: entity)
        return Collection(entity: entity, trait: collectionTrait, parents: parents)
    }
}
