import UIKit
import Exocore

class CollectionViewController: UIViewController, EntityTraitView {
    private let objectsStoryboard: UIStoryboard = UIStoryboard(name: "Objects", bundle: nil)

    private var entity: EntityExt!
    private var collection: TraitInstance<Exomind_Base_V1_Collection>!
    private var entityListViewController: EntityListViewController!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance, fullEntity: Bool) {
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
        self.entityListViewController = (objectsStoryboard.instantiateViewController(withIdentifier: "EntityListViewController") as! EntityListViewController)
        self.addChild(self.entityListViewController)
        self.view.addSubview(self.entityListViewController.view)
        self.entityListViewController.didMove(toParent: self)
        self.entityListViewController.viewWillAppear(false)
        self.entityListViewController.viewDidAppear(false)

        self.entityListViewController.itemClickHandler = { [weak self] (entity) in
            self?.handleItemClick(entity)
        }

        self.entityListViewController.collectionClickHandler = { [weak self] (entity, collection) in
            self?.handleItemClick(collection)
        }

        self.entityListViewController.actionsForEntity = { [weak self] entity in
            guard let this = self else {
                return []
            }
            let navController = this.navigationController as? NavigationController
            return Actions.forEntity(entity, parentId: this.entity.id, navController: navController)
        }

        self.entityListViewController.actionsForSelectedEntities = { [weak self] entities in
            guard let this = self else {
                return []
            }
            let navController = this.navigationController as? NavigationController
            return Actions.forSelectedEntities(entities, parentId: this.entity.id, navController: navController)
        }

        self.entityListViewController.loadData(fromChildrenOf: self.entity.id)
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        self.setupNavigationActions()
    }

    private func setupNavigationActions() {
        guard let nav = self.navigationController as? NavigationController else {
            return
        }

        nav.resetState()
        nav.setBarActions([
            NavigationControllerBarAction(icon: .search, handler: { [weak self] () -> Void in
                self?.handleShowSearch()
            }),
            NavigationControllerBarAction(icon: .checkCircle, handler: { [weak self] () -> Void in
                self?.entityListViewController.editMode = !(self?.entityListViewController.editMode ?? true)
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
            textField.text = self?.collection.message.name
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
        (self.navigationController as? NavigationController)?.showCreateObject(self.entity.id) { [weak self] (res) -> Void in
            guard case let .successCreated(entity) = res, let entity = entity else {
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

    deinit {
        print("CollectionViewController > Deinit")
    }
}
