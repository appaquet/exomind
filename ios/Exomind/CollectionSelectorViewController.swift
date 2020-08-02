import UIKit
import Exocore

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

    private var collectionsQuery: ExpandableQuery?
    private var collectionsQueryFilter: String?

    private var entityQuery: QueryStreamHandle?
    private var entityComplete: EntityExt?
    private var entityParentsQuery: QueryStreamHandle?
    private var entityParents: [EntityExt]?

    private var currentFilter: String?
    private var searchDebouncer: Debouncer!
    private var collectionsData: [EntityExt] = []

    override func viewDidLoad() {
        super.viewDidLoad()

        self.tableView.delegate = self
        self.tableView.keyboardDismissMode = .onDrag

        self.searchBar = UISearchBar()
        self.navigationItem.titleView = self.searchBar
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

            let collectionsEntities = collectionsResults.map({ $0.entity.toExtension() })
            let combined = entityParents + collectionsEntities
            var uniqueMap = [String: EntityExt]()
            self.collectionsData = combined
                    .filter { (entity: EntityExt) -> Bool in
                if uniqueMap[entity.id] == nil {
                    uniqueMap[entity.id] = entity
                    return true
                } else {
                    return false
                }
            }
            self.tableView.reloadData()
        } else {
            self.collectionsData = []
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

            DispatchQueue.main.async {
                this.loadData()
            }
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
        self.collectionsQuery = ExpandableQuery(query: collectionsQuery.build(), onChange: { [weak self] in
            DispatchQueue.main.async {
                self?.loadData()
            }
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
                self?.entityParents = res?.entities.map({ $0.entity.toExtension() })
                DispatchQueue.main.async {
                    self?.loadData()
                }
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
        self.collectionsData.count
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = self.tableView.dequeueReusableCell(withIdentifier: "cell", for: indexPath)

        let collectionEntity = self.collectionsData[(indexPath as NSIndexPath).item]
        let collectionTrait = collectionEntity.priorityTrait

        cell.textLabel?.text = collectionTrait?.displayName ?? "*INVALID*"
        cell.imageView?.image = collectionTrait.map {
            ObjectsIcon.icon(forAnyTrait: $0, color: UIColor.label, dimension: 24)
        }

        if (self.hasParent(parentEntityId: collectionEntity.id)) {
            cell.accessoryType = .checkmark
        } else {
            cell.accessoryType = .none
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let cell = self.tableView.cellForRow(at: indexPath)!
        let collectionEntity = self.collectionsData[(indexPath as NSIndexPath).item]
        if (cell.accessoryType == .checkmark) {
            cell.accessoryType = .none
            self.removeParent(parentEntityId: collectionEntity.id)
        } else {
            self.addParent(parentEntityId: collectionEntity.id)
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

        return Mutations.hasParent(entity: entityComplete, parentId: id)
    }

    private func addParent(parentEntityId id: String) {
        guard let entityComplete = self.entityComplete else {
            return
        }

        do {
            try Mutations.addParent(entity: entityComplete, parentId: id)
        } catch {
            print("CollectionSelectionViewController> Error adding parent \(error)")
        }
    }

    private func removeParent(parentEntityId id: String) {
        guard let entityComplete = self.entityComplete else {
            return
        }

        Mutations.removeParent(entity: entityComplete, parentId: id)
    }

    deinit {
        print("CollectionSelectionViewController > Deinit")
    }
}
