import UIKit
import Exocore

class SearchViewController: NavigationController {
    var fromEntityId: EntityId?
    var selectionHandler: ((EntityExt) -> Void)?

    private var doneButton: UIBarButtonItem!

    override func viewDidLoad() {
        super.viewDidLoad()

        // set colors of navigation bar
        Stylesheet.styleNavigationBar(self.navigationBar, bgColor: Stylesheet.searchNavigationBarBg, fgColor: Stylesheet.searchNavigationBarFg)

        let containerVc = (self.topViewController as! SearchCollectionContainer)
        containerVc.fromEntityId = self.fromEntityId
        containerVc.selectionHandler = self.selectionHandler
        containerVc.searchNavigationController = self

        // keep the done button to add it to other views
        self.doneButton = self.topViewController?.navigationItem.rightBarButtonItem
    }

    override func setBarActions(_ actions: [NavigationControllerBarAction]) {
        // we don't allow any custom navigation buttons. Only the done button is added
    }

    override func resetState() {
        super.resetState()

        // we always add the done button
        self.topViewController?.navigationItem.rightBarButtonItem = doneButton
    }

    deinit {
        print("SearchViewController > Deinit")
    }
}

class SearchCollectionContainer: UIViewController, UISearchBarDelegate {
    private let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    fileprivate var fromEntityId: String?
    fileprivate var selectionHandler: ((EntityExt) -> Void)?
    fileprivate weak var searchNavigationController: SearchViewController!

    private var searchBar: UISearchBar!
    private var childrenViewController: ChildrenViewController!
    private var searchText: String?
    private var searchDebouncer: Debouncer!

    override func viewDidLoad() {
        super.viewDidLoad()

        self.searchDebouncer = Debouncer(timeInterval: 0.5) { [weak self] in
            self?.executeQuery()
        }

        self.setupChildrenViewController()
        self.searchBar = UISearchBar()
        self.navigationItem.titleView = self.searchBar
        self.searchBar.becomeFirstResponder()
        self.searchBar.placeholder = "Search"
        self.searchBar.delegate = self
    }

    private func setupChildrenViewController() {
        self.childrenViewController = (self.mainStoryboard.instantiateViewController(withIdentifier: "ChildrenViewController") as! ChildrenViewController)
        self.childrenViewController.tableView.keyboardDismissMode = .onDrag

        self.addChild(self.childrenViewController)
        self.view.addSubview(self.childrenViewController.view)

        self.childrenViewController.setItemClickHandler { [weak self] (entity) -> Void in
            self?.handleItemSelection(entity)
        }

        self.childrenViewController.setSwipeActions([
            ChildrenViewSwipeAction(action: .inbox, color: Stylesheet.collectionSwipeDoneBg, state: .state1, mode: .exit, handler: { [weak self] (entity) -> Void in
                self?.handleCopyInbox(entity)
            })
        ])

        self.childrenViewController.loadData(withResults: [])
    }

    override func viewWillAppear(_ animated: Bool) {
        (self.navigationController as? NavigationController)?.resetState()
    }

    override func viewDidAppear(_ animated: Bool) {
        self.searchBar.becomeFirstResponder()
    }

    @IBAction func handleDoneClick(_ sender: AnyObject) {
        self.dismiss(animated: true, completion: nil)
    }

    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        self.searchText = searchText
        self.searchDebouncer.renewInterval()
    }

    private func executeQuery() {
        guard let searchText = self.searchText else {
            return
        }

        let query = QueryBuilder
                .matching(query: searchText)
                .count(30)
                .build()
        self.childrenViewController.loadData(fromQuery: query)
    }

    private func handleItemSelection(_ entity: EntityExt) {
        self.searchBar.resignFirstResponder() // prevent keyboard from transitioning weirdly
        if let handler = self.selectionHandler {
            handler(entity)
        } else {
            self.searchNavigationController.pushObject(.entity(entity: entity))
        }
    }

    private func handleCopyInbox(_ entity: EntityExt) {
        let inboxRelation = entity
                .traitsOfType(Exomind_Base_CollectionChild.self)
                .first(where: { $0.message.collection.entityID == "inbox" })
        let traitId = inboxRelation?.id ?? "child_inbox"

        var child = Exomind_Base_CollectionChild()
        child.collection.entityID = "inbox"
        child.weight = UInt64(Date().millisecondsSince1970)

        do {
            let mutation = try MutationBuilder
                    .updateEntity(entityId: entity.id)
                    .putTrait(message: child, traitId: traitId)
                    .build()

            ExocoreClient.store.mutate(mutation: mutation)
        } catch {
            print("SearchCollectionContainer > Error copying to inbox: \(error)")
        }
    }

    deinit {
        print("SearchCollectionContainer > Deinit")
    }
}
