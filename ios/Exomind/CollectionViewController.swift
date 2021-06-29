import UIKit
import Exocore

class CollectionViewController: UIViewController, EntityTraitView {
    private let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    private var entity: EntityExt!
    private var collection: TraitInstance<Exomind_Base_Collection>!
    private var entityListViewController: EntityListViewController!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance) {
        self.entity = entity
        self.collection = entity.trait(withId: trait.id)
        self.navigationItem.title = trait.displayName
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        self.setupEntityList()
        self.setupNavigationActions()
    }

    private func setupEntityList() {
        self.entityListViewController = (mainStoryboard.instantiateViewController(withIdentifier: "EntityListViewController") as! EntityListViewController)
        self.addChild(self.entityListViewController)
        self.view.addSubview(self.entityListViewController.view)

        self.entityListViewController.setItemClickHandler { [weak self] in
            self?.handleItemClick($0)
        }

        self.entityListViewController.setSwipeActions([
            EntityListSwipeAction(action: .check, color: Stylesheet.collectionSwipeDoneBg, side: .leading, style: .destructive, handler: { [weak self] (entity, callback) -> Void in
                self?.handleDone(entity)
                callback(true)
            }),
            EntityListSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, side: .trailing, style: .normal, handler: { [weak self] (entity, callback) -> Void in
                self?.handleAddToCollection(entity)
                callback(false)
            }),
            EntityListSwipeAction(action: .clock, color: Stylesheet.collectionSwipeLaterBg, side: .trailing, style: .normal, handler: { [weak self] (entity, callback) -> Void in
                self?.handleMoveLater(entity, callback: callback)
            }),
        ])

        self.entityListViewController.loadData(fromChildrenOf: self.entity.id)
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        self.setupNavigationActions()
    }

    private func setupNavigationActions() {
        let nav = (self.navigationController as! NavigationController)
        nav.resetState()
        nav.setBarActions([
            NavigationControllerBarAction(icon: .search, handler: { [weak self] () -> Void in
                self?.handleShowSearch()
            })
        ])

        // quick button only visible in current
        nav.setQuickButtonActions([
            QuickButtonAction(icon: .iCursor, handler: { [weak self] () -> Void in
                self?.handleCollectionRename()
            }),
            QuickButtonAction(icon: .plus, handler: { [weak self] () -> Void in
                self?.handleCreateObject()
            }),
            QuickButtonAction(icon: .folderOpen, handler: { [weak self] () -> Void in
                self?.handleAddToCollection()
            })
        ])
    }

    private func handleCollectionRename() {
        let alert = UIAlertController(title: "Name", message: "Enter a new name", preferredStyle: UIAlertController.Style.alert)
        alert.addTextField(configurationHandler: { [weak self] (textField: UITextField!) in
            textField.text = self?.collection.displayName
            textField.isSecureTextEntry = false
        })
        alert.addAction(UIAlertAction(title: "Ok", style: .default, handler: { [weak self] (alertAction) -> Void in
            let newName = alert.textFields![0] as UITextField

            guard let entity = self?.entity,
                  let collection = self?.collection,
                  let name = newName.text else {
                return
            }

            var newCollection = collection.message
            newCollection.name = name

            do {
                let mutation = try MutationBuilder
                        .updateEntity(entityId: entity.id)
                        .putTrait(message: newCollection, traitId: collection.id)
                        .build()
                ExocoreClient.store.mutate(mutation: mutation)
            } catch {
                print("CollectionViewController> Couldn't rename \(error)")
            }
        }))
        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel, handler: nil))
        self.present(alert, animated: true, completion: nil)
    }

    private func handleCreateObject() -> ()? {
        (self.navigationController as? NavigationController)?.showCreateObject(self.entity.id) { [weak self] (entity) -> Void in
            guard let entity = entity else {
                return
            }
            (self?.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
        }
    }

    private func handleAddToCollection() {
        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: self.entity)
    }

    private func handleShowSearch() {
        (self.navigationController as? NavigationController)?.showSearch(self.entity.id)
    }

    private func handleItemClick(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
    }

    private func handleDone(_ entity: EntityExt) {
        ExomindMutations.removeParent(entity: entity, parentId: self.entity.id)
    }

    private func handleMoveLater(_ entity: EntityExt, callback: @escaping (Bool) -> Void) {
        (self.navigationController as? NavigationController)?.showTimeSelector(forEntity: entity, callback: callback)
    }

    private func handleAddToCollection(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: entity)
    }

    deinit {
        print("CollectionViewController > Deinit")
    }
}
