import UIKit
import Exocore

class EntityViewController: UIViewController {
    fileprivate let objectsStoryboard: UIStoryboard = UIStoryboard(name: "Objects", bundle: nil)

    private var entityId: EntityId!
    private var entity: EntityExt?
    private var entityTrait: AnyTraitInstance?
    private var fullEntity: Bool = false
    private var specificTraitId: TraitId?

    private var entityQuery: QueryStreamHandle?
    private var viewCreated: Bool = false

    private var entityViewController: EntityTraitView?

    func populate(entity: EntityExt) {
        self.entityId = entity.id
        self.entity = entity
        self.entityTrait = entity.priorityTrait
        self.fetchFullEntity()
    }

    func populate(entityTrait: AnyTraitInstance) {
        self.entity = entityTrait.entity
        self.entityId = entityTrait.entity?.id
        self.specificTraitId = entityTrait.trait.id
        self.fetchFullEntity()
    }

    func populate(entityId: EntityId) {
        self.entityId = entityId
        self.fetchFullEntity()
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.renderEntity()
    }

    func fetchFullEntity() {
        let query = QueryBuilder.withId(self.entityId).build()
        self.entityQuery = ExocoreClient.store.watchedQuery(query: query, onChange: { [weak self] (status, res) in
            guard let this = self,
                  let res = res,
                  let entity = res.entities.first?.entity else {
                return
            }

            this.fullEntity = true
            this.entity = entity.toExtension()
            if let specificTraitId = this.specificTraitId {
                this.entityTrait = this.entity?.trait(anyWithId: specificTraitId)
            } else {
                this.entityTrait = this.entity?.priorityTrait
            }

            DispatchQueue.main.async { [weak this] in
                this?.renderEntity()
            }
        })
    }

    func renderEntity() {
        guard let entity = self.entity,
              let trait = self.entityTrait else {
            return
        }

        self.title = trait.displayName

        if !self.viewCreated {
            self.viewCreated = true
            self.createEntityViewController()
        } else if let vc = self.entityViewController {
            vc.loadEntityTrait(entity: entity, trait: trait, fullEntity: self.fullEntity)
        }
    }

    func createEntityViewController() {
        guard let entity = self.entity,
              let trait = self.entityTrait else {
            return
        }

        let traitType = trait.type

        var vc: EntityTraitView
        switch traitType {
        case .emailThread:
            vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "EmailThreadViewController") as! EmailThreadViewController
        case .draftEmail:
            vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "DraftEmailViewController") as! DraftEmailViewController
        case .email:
            vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "EmailViewController") as! EmailViewController
        case .collection, .favorites:
            vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "CollectionViewController") as! CollectionViewController
        case .note:
            vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "NoteViewController") as! NoteViewController
        case .task:
            vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "TaskViewController") as! TaskViewController
        case .link:
            vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "LinkViewController") as! LinkViewController
        default:
            return
        }

        vc.loadEntityTrait(entity: entity, trait: trait, fullEntity: self.fullEntity)
        self.entityViewController = vc
        self.showVC(vc)
    }

    override func viewWillAppear(_ animated: Bool) {
        // reset states in navigation controller (buttons, etc.)
        (self.navigationController as? NavigationController)?.resetState()
    }

    func showVC(_ vc: EntityTraitView) {
        self.addChild(vc)
        vc.view.frame = CGRect(x: 0, y: 0, width: self.view.frame.size.width, height: self.view.frame.size.height);
        self.view.addSubview(vc.view)
        vc.didMove(toParent: self)
        vc.viewWillAppear(false)
        vc.viewDidAppear(false)
    }

    deinit {
        print("EntityViewController > Deinit")
    }
}

protocol EntityTraitView: UIViewController {
    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance, fullEntity: Bool)
}
