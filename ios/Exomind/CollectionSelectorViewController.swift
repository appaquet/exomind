//
//  CollectionSelectorViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-11-27.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit

class CollectionSelectorViewController: UINavigationController {
    var forEntity: HCEntity!
    var tableView: CollectionSelectorTableViewController!

    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView = (super.topViewController as! CollectionSelectorTableViewController)
        self.tableView.entity = forEntity

        // set colors of navigation bar
        Stylesheet.styleNavigationBar(self.navigationBar, bgColor: Stylesheet.collectionSelectorNavigationBarBg, fgColor: Stylesheet.collectionSelectorNavigationBarFg)
    }
}

class CollectionSelectorTableViewController: UITableViewController, UISearchBarDelegate {
    fileprivate var querySet: QuerySet!
    fileprivate var collectionsQuery: Query!
    fileprivate var entityQuery: Query!
    fileprivate var parentsQuery: Query?
    fileprivate var entity: HCEntity!

    fileprivate var collectionsData: [HCEntity]!

    fileprivate var searchBar: UISearchBar!
    fileprivate var currentFilter: String?
    fileprivate var expandPending = false

    override func viewDidLoad() {
        super.viewDidLoad()

        self.tableView.delegate = self
        self.tableView.keyboardDismissMode = .onDrag

        self.searchBar = UISearchBar()
        self.navigationItem.titleView = self.searchBar
        self.searchBar.placeholder = "Filter"
        self.searchBar.delegate = self
        
        self.loadData()
    }

    func loadData() {
        if (self.querySet == nil) {
            self.querySet = DomainStore.instance.getQuerySet()
            self.querySet.onChange { [weak self] () -> () in
                self?.loadData()
            }
        }
        
        let oldCollectionsQuery = self.collectionsQuery
        if let keywords = self.currentFilter {
            self.collectionsQuery = self.querySet.executeQuery(
                HCQueries.Entities()
                    .withTrait(CollectionSchema.fullType) { tb in
                        tb.whereFieldMatch("name", value: keywords)
                    }
                    .sortBy("-score")
            )
        } else {
            self.collectionsQuery = self.querySet.executeQuery(HCQueries.Entities().withTrait(CollectionSchema.fullType))
        }

        if oldCollectionsQuery?.hash() != self.collectionsQuery.hash() {
            oldCollectionsQuery?.release()
            self.expandPending = false
        }

        self.entityQuery = self.querySet.executeQuery(HCQueries.Entities().withEntityId(self.entity.id))
        if let entity = self.entityQuery.resultAsEntity() {
            self.entity = entity
            let parents = ExomindDSL.on(entity).relations.getParents().map { $0.to }
            self.parentsQuery = self.querySet.executeQuery(HCQueries.Entities().withEntityIds(parents))
        }

        // if everything is loaded
        if let parentsQuery = self.parentsQuery?.resultsAsEntities(),
            let _ = self.entity,
            self.collectionsQuery.isLoaded(),
            self.parentsQuery?.isLoaded() ?? false {
            
            let specialEntities = [SessionStore.inboxEntity(), SessionStore.mindEntity()].compactMap { $0 }
            let combined = parentsQuery + specialEntities + self.collectionsQuery.resultsAsEntities()
            var uniqueMap = [String:HCEntity]()
            self.collectionsData = combined.filter { entity in
                if uniqueMap[entity.id] == nil {
                    uniqueMap[entity.id] = entity
                    return true
                } else {
                    return false
                }
            }
            self.tableView.reloadData()
        }
    }

    var debouncedSearch: (()->())?
    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        if (!searchText.isEmpty) {
            self.currentFilter = searchText
        } else {
            self.currentFilter = nil
        }

        // TODO:
//        if self.debouncedSearch == nil {
//            self.debouncedSearch = Debouncer.debounce(delay: 500, queue: DispatchQueue.main, action: { [weak self] in
//                self?.loadData()
//            })
//        }
//        self.debouncedSearch?()
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        if (self.collectionsQuery.isLoaded()) {
            return self.collectionsData.count
        } else {
            return 0
        }
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = self.tableView.dequeueReusableCell(withIdentifier: "cell", for: indexPath)

        let collectionEntity = self.collectionsData[(indexPath as NSIndexPath).item]
        let entityTrait = EntityTraitOld(entity: collectionEntity)
        
        cell.textLabel!.text = entityTrait?.displayName
        cell.imageView?.image = entityTrait.map { ObjectsIcon.icon(forEntityTrait: $0, color: UIColor.black, dimension: 24) }

        let currentlyParent = ExomindDSL.on(self.entity).relations.hasParent(parentId: collectionEntity.id)
        if (currentlyParent) {
            cell.accessoryType = .checkmark
        } else {
            cell.accessoryType = .none
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let cell = self.tableView.cellForRow(at: indexPath)!
        let collection = self.collectionsData[(indexPath as NSIndexPath).item]
        if (cell.accessoryType == .checkmark) {
            cell.accessoryType = .none
            ExomindDSL.on(self.entity).relations.removeParent(parentId: collection.id)
        } else {
            ExomindDSL.on(self.entity).relations.addParent(parentId: collection.id)
            cell.accessoryType = .checkmark
        }
    }
    
    override func scrollViewDidScroll(_ scrollView: UIScrollView) {
        let totalHeight = (scrollView.contentSize.height - tableView.frame.size.height)
        let currentPosition = scrollView.contentOffset.y
        let averageHeight = CGFloat(74)
        let itemsComingUp = (totalHeight - currentPosition) / averageHeight
        
        // if we suddenly have more items, that means we are at beginning or expansion worked
        if (itemsComingUp > 10) {
            self.expandPending = false
        }
        
        // if only 5 items or less are coming up, we load new
        if (itemsComingUp < 10 && self.collectionsQuery.isLoaded() && !self.expandPending) {
//            self.expandPending = true
//            DispatchQueue.main.async(execute: { [weak self] () -> Void in
//                if let this = self, let query = this.collectionsQuery.expand() {
//                    print("CollectionSelectorViewController > Expanding query...")
//                    this.collectionsQuery = this.querySet.executeQuery(query, reExecute: true)
//                }
//            })
        }
    }

    @IBAction func handleDoneClick(_ sender: AnyObject) {
        self.dismiss(animated: true, completion: nil)
    }


    deinit {
        print("CollectionSelectionViewController > Deinit")
    }
}
