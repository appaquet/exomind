import UIKit
import Exocore
import SwiftUI

class CollectionSelectorViewController: UINavigationController {
    var forEntity: EntityExt!
    var tableView: CollectionSelectorTableViewController!

    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView = (super.topViewController as! CollectionSelectorTableViewController)
        self.tableView.partialEntity = forEntity

        // set colors of navigation bar
        Stylesheet.styleNavigationBar(self.navigationBar, bgColor: Stylesheet.collectionSelectorNavigationBarBg, fgColor: Stylesheet.collectionSelectorNavigationBarFg)
    }
}

class CollectionSelectorTableViewController: UITableViewController, UISearchBarDelegate {
    fileprivate var partialEntity: EntityExt!

    private var searchBar: UISearchBar!

    private var collectionsQuery: ManagedQuery?
    private var collectionsQueryFilter: String?

    private var entityQuery: QueryStreamHandle?
    private var entityComplete: EntityExt?
    private var entityParentsQuery: QueryStreamHandle?
    private var entityParents: [Collection]?

    private var currentFilter: String?
    private var searchDebouncer: Debouncer!
    private var collectionData: [Collection] = []

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
        if self.entityQuery == nil {
            self.queryEntity()
        }

        if self.collectionsQuery == nil || self.collectionsQueryFilter != self.currentFilter {
            self.queryFilteredCollections()
        }

        if let collectionsResults = self.collectionsQuery?.results,
           let entityParents = self.entityParents {

            let collectionsEntities: [Collection] = collectionsResults.compactMap({ res in
                Collection.fromEntity(entity: res.entity.toExtension())
            })
            let combined = entityParents + collectionsEntities
            var uniqueMap = [String: Collection]()
            self.collectionData = combined.filter { (col: Collection) -> Bool in
                if uniqueMap[col.entity.id] == nil {
                    uniqueMap[col.entity.id] = col
                    return true
                } else {
                    return false
                }
            }

            DispatchQueue.main.async {
                self.tableView.reloadData()
            }
        } else {
            self.collectionData = []
        }
    }

    private func queryEntity() {
        let entityQuery = QueryBuilder.withId(self.partialEntity.id).build()
        self.entityQuery = ExocoreClient.store.watchedQuery(query: entityQuery, onChange: { [weak self] (status, res) in
            guard let this = self,
                  res?.entities.count ?? 0 > 0 else {
                return
            }

            let entity = res!.entities[0].entity.toExtension()
            this.entityComplete = entity
            this.queryEntityParents(entity: entity)
            this.loadData()
        })
    }

    private func queryFilteredCollections() {
        var collectionsQuery: QueryBuilder;
        if let currentFilter = currentFilter {
            let traitQuery = TraitQueryBuilder.matching(query: currentFilter).build()
            collectionsQuery = QueryBuilder.withTrait(Exomind_Base_Collection.self, query: traitQuery)
        } else {
            collectionsQuery = QueryBuilder.withTrait(Exomind_Base_Collection.self)
        }
        collectionsQuery = collectionsQuery.count(30)
        self.collectionsQuery = ManagedQuery(query: collectionsQuery.build(), onChange: { [weak self] in
            self?.loadData()
        })
        self.collectionsQueryFilter = currentFilter
    }

    private func queryEntityParents(entity: EntityExt) {
        let parents = entity
                .traitsOfType(Exomind_Base_CollectionChild.self)
                .map({ $0.message.collection.entityID })

        if !parents.isEmpty {
            let query = QueryBuilder.withIds(parents).count(100).build()
            self.entityParentsQuery = ExocoreClient.store.watchedQuery(query: query, onChange: { [weak self] (status, res) in
                self?.entityParents = res?.entities.compactMap({ Collection.fromEntity(entity: $0.entity.toExtension()) })
                self?.loadData()
            })
        } else {
            self.entityParents = []
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
        let collection = self.collectionData[(indexPath as NSIndexPath).item]

        let name = collection.trait.strippedDisplayName
        let checked = self.hasParent(parentEntityId: collection.entity.id)
        let img = ObjectsIcon.icon(forAnyTrait: collection.trait, color: UIColor.label, dimension: CollectionSelectorCell.ICON_SIZE)
        let parents = Collections.instance.entityParentsPillData(entity: collection.entity)

        let cellData = CollectionSelectorCellData(id: collection.entity.id, name: name, checked: checked, icon: img, parents: parents)

        let cell = tableView.dequeueReusableCell(withIdentifier: "cell", for: indexPath) as! SwiftUICellViewHost<CollectionSelectorCell>
        cell.setView(view: CollectionSelectorCell(data: cellData), parentController: self)

        return cell
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        self.tableView.deselectRow(at: indexPath, animated: false)

        let collection = self.collectionData[(indexPath as NSIndexPath).item]
        if (self.hasParent(parentEntityId: collection.entity.id)) {
            self.removeParent(parentEntityId: collection.entity.id)
        } else {
            self.addParent(parentEntityId: collection.entity.id)
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
        guard let entityComplete = self.entityComplete else {
            return false
        }

        return ExomindMutations.hasParent(entity: entityComplete, parentId: id)
    }

    private func addParent(parentEntityId id: String) {
        guard let entityComplete = self.entityComplete else {
            return
        }

        do {
            try ExomindMutations.addParent(entity: entityComplete, parentId: id)
        } catch {
            print("CollectionSelectionViewController> Error adding parent \(error)")
        }
    }

    private func removeParent(parentEntityId id: String) {
        guard let entityComplete = self.entityComplete else {
            return
        }

        ExomindMutations.removeParent(entity: entityComplete, parentId: id)
    }

    deinit {
        print("CollectionSelectionViewController > Deinit")
    }
}

fileprivate struct Collection {
    let entity: EntityExt
    let trait: TraitInstance<Exomind_Base_Collection>
    let parents: [CollectionPillData]

    static func fromEntity(entity: EntityExt) -> Collection? {
        guard let collectionTrait = entity.traitOfType(Exomind_Base_Collection.self) else {
            return nil
        }
        let parents = Collections.instance.entityParentsPillData(entity: entity)
        return Collection(entity: entity, trait: collectionTrait, parents: parents)
    }
}
