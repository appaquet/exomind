import UIKit
import Exocore

class CollectionViewController: UIViewController, EntityTraitView {
    private let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    private var entity: EntityExt!
    private var trait: AnyTraitInstance!
    private var childrenVC: ChildrenViewController!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance) {
        self.entity = entity
        self.trait = trait
        self.title = trait.displayName
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        self.setupChildrenVC()
        self.setupNavigationActions()
    }

    private func setupChildrenVC() {
        self.childrenVC = (mainStoryboard.instantiateViewController(withIdentifier: "ChildrenViewController") as! ChildrenViewController)
        self.addChild(self.childrenVC)
        self.view.addSubview(self.childrenVC.view)

        self.childrenVC.setItemClickHandler { [weak self] in
            self?.handleItemClick($0)
        }

        self.childrenVC.setSwipeActions([
            ChildrenViewSwipeAction(action: .check, color: Stylesheet.collectionSwipeDoneBg, state: .state1, mode: .exit, handler: { [weak self] (entity) -> Void in
                self?.handleDone(entity)
            }),
            ChildrenViewSwipeAction(action: .clock, color: Stylesheet.collectionSwipeLaterBg, state: .state3, mode: .switch, handler: { [weak self] (entity) -> Void in
                self?.handleMoveLater(entity)
            }),
            ChildrenViewSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, state: .state4, mode: .switch, handler: { [weak self] (entity) -> Void in
                self?.handleAddToCollection(entity)
            })
        ])

        self.childrenVC.loadData(fromChildrenOf: self.entity.id)
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
//        let alert = UIAlertController(title: "Name", message: "Enter a new name", preferredStyle: UIAlertController.Style.alert)
//        alert.addTextField(configurationHandler: { [weak self] (textField: UITextField!) in
//            textField.text = self?.trait.displayName
//            textField.isSecureTextEntry = false
//        })
//        alert.addAction(UIAlertAction(title: "Ok", style: .default, handler: { [weak self] (alertAction) -> Void in
//            guard let this = self else {
//                return
//            }
//            let newName = alert.textFields![0] as UITextField
//
//            if let collection = this.trait.trait as? CollectionFull, let name = newName.text {
//                collection.name = name
//                ExomindDSL.on(this.trait.entity).mutate.update(collection).execute()
//            }
//        }))
//        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel, handler: nil))
//        self.present(alert, animated: true, completion: nil)
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
//        guard let entityTrait = self.trait else {
//            return
//        }
//        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: entityTrait.entity)
    }

    private func handleShowSearch() {
        (self.navigationController as? NavigationController)?.showSearch(self.entity.id)
    }

    private func handleItemClick(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
    }

    private func handleDone(_ entity: EntityExt) {
        guard let childTrait = entity
                .traitsOfType(Exomind_Base_CollectionChild.self)
                .first(where: { $0.message.collection.entityID == self.entity.id }) else {
            return
        }

        let mutation = MutationBuilder
                .updateEntity(entityId: entity.id)
                .deleteTrait(traitId: childTrait.id)
                .build()
        ExocoreClient.store.mutate(mutation: mutation)
    }

    private func handleMoveLater(_ entity: EntityExt) {
//   TODO:     (self.navigationController as? NavigationController)?.showTimeSelector(forEntity: entity)
    }

    private func handleAddToCollection(_ entity: EntityExt) {
// TODO:        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: entity)
    }

    deinit {
        print("CollectionViewController > Deinit")
    }
}
